//! Fn trait implementations

use quote::{Ident, Tokens};
use codegen::operations::match_values;

static TRAITS: [(&str, &str, &str); 3] = [
    ("FnOnce", "call_once", ""),
    ("FnMut", "call_mut", "&mut"),
    ("Fn", "call", "&"),
];

pub fn impl_fn() -> Vec<Tokens> {
    (1..9u32)
        .flat_map(|size| {
            TRAITS.into_iter()
                .enumerate()
                .map(move |(index, &(trt, func, prefix))| {
                    let trt = Ident::from(trt);
                    let func = Ident::from(func);

                    let deref = Ident::from(prefix.replace("&", "ref "));
                    let prefix = Ident::from(prefix);

                    let idx1 = 0..size;
                    let idx2 = 0..size;

                    let types1: Vec<_> = (0..size).map(|idx| Ident::from(format!("T{}", idx))).collect();
                    let types2 = types1.clone();
                    let types3 = types1.clone();

                    let args1: Vec<_> = (0..size).map(|idx| Ident::from(format!("args_{}", idx))).collect();
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
                            let sink = graph.add_node(Node::Call(self.func));
                            #( graph.add_edge(#args3, sink, #idx1); )*

                            drop(graph);
                            
                            let args = (
                                #( self.parameter(#idx2), )*
                            );

                            let #prefix Function { #deref module, func, #deref thunk } = self;
                            let res = thunk.#func(args);
                            Self::ret_impl(&module, func, res);

                            sink
                        },
                    );

                    quote! {
                        #[cfg(feature = "functions")]
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
