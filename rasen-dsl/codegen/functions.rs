//! Fn trait implementations

use codegen::operations::match_values;
use proc_macro2::{Ident, Span, TokenStream};

static TRAITS: [(&str, &str, Option<&str>); 3] = [
    ("FnOnce", "call_once", None),
    ("FnMut", "call_mut", Some("&mut")),
    ("Fn", "call", Some("&")),
];

pub fn impl_fn() -> Vec<TokenStream> {
    (1..9u32)
        .flat_map(|size| {
            TRAITS.into_iter()
                .enumerate()
                .map(move |(index, &(trt, func, prefix))| {
                    let trt = Ident::new(&trt, Span::call_site());
                    let func = Ident::new(&func, Span::call_site());

                    let deref =  prefix.map(|prefix| -> TokenStream {
                        prefix.replace("&", "ref ").parse().unwrap()
                    });

                    let prefix = prefix.map(|prefix| -> TokenStream {
                        prefix.parse().unwrap()
                    });

                    let idx1 = 0..size;
                    let idx2 = 0..size;

                    let types1: Vec<_> = (0..size).map(|idx| Ident::new(&format!("T{}", idx), Span::call_site())).collect();
                    let types2 = types1.clone();
                    let types3 = types1.clone();

                    let args1: Vec<_> = (0..size).map(|idx| Ident::new(&format!("args_{}", idx), Span::call_site())).collect();
                    let args2 = args1.clone();
                    let args3 = args1.clone();

                    let tuple = quote! { ( #( Value<#types1>, )* ) };
                    let output = if index == 0 {
                        quote! { type Output = <T as FnOnce<#tuple>>::Output; }
                    } else {
                        quote!()
                    };

                    let func_impl = match_values(
                        &args1,
                        &quote! {
                            self.thunk.#func(( #( Value::Concrete( #args2 ), )* ))
                        },
                        quote! {
                            use std::sync::atomic::Ordering;
                            
                            let sink = graph.add_node(Node::Call(self.func));
                            #( graph.add_edge(#args3, sink, #idx1); )*
                            drop(graph);

                            if !self.built.swap(true, Ordering::SeqCst) {
                                let args = (
                                    #( self.parameter(#idx2), )*
                                );

                                let #prefix Function { #deref module, func, #deref thunk, .. } = self;
                                let res = thunk.#func(args);
                                Self::ret_impl(&module, func, res);
                            }

                            sink
                        },
                    );

                    quote! {
                        impl<T, R: IntoValue + Clone, #( #types2: IntoValue + Clone ),*> #trt<#tuple> for Function<T> where T: #trt<#tuple, Output=Value<R>>, #( Function<T>: Parameter<#types3> ),* {
                            #output
                            extern "rust-call" fn #func(#prefix self, ( #( #args1, )* ): #tuple) -> Value<R> {
                                #func_impl
                            }
                        }
                    }
                })
        })
        .collect()
}
