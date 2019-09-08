//! Fn trait implementations

use proc_macro2::{Ident, Literal, Span, TokenStream};

pub fn impl_fn() -> Vec<TokenStream> {
    (1..9u32)
        .map(|size| {
            let generics: Vec<_> = (0..size).map(|index| Ident::new(&format!("T{}", index), Span::call_site())).collect();
            let values: Vec<_> = generics.iter().map(|ty| quote! { Value<Parse, #ty>, }).collect();
            let types: Vec<_> = generics.iter().map(|ty| quote! { #ty: AsTypeName, }).collect();
            let containers: Vec<_> = generics.iter().map(|ty| quote! { Container<#ty, Value = ParseNode> }).collect();

            let create: Vec<_> = generics.iter()
                .enumerate()
                .map(|(index, ty)| {
                    let index = index as u32;
                    quote! {
                        with_graph(|graph| Value(graph.add_node(Node::Parameter(#index, #ty::TYPE_NAME)))),
                    }
                })
                .collect();

            let edges: Vec<_> = (0..size)
                .map(|index| {
                    let field = Literal::u32_unsuffixed(index);
                    quote! {
                        graph.add_edge((self.#field).0, node, #index);
                    }
                })
                .collect();

            quote! {
                impl<#( #generics ),*> FnArgs for (#( #values )*) where #( #types )* Parse: #( #containers )+* {
                    fn create() -> Self {
                        ( #( #create )* )
                    }

                    fn call(self, func: FunctionRef) -> ParseNode {
                        with_graph(|graph| {
                            let node = graph.add_node(Node::Call(func));
                            #( #edges )*
                            node
                        })
                    }
                }
            }
        })
        .collect()
}
