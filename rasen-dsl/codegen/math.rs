//! Mul trait implementation

use codegen::{
    defs::{all_types, Category, Type},
    mul::impl_mul_variant,
};
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};

fn impl_math_variant(
    (trait_id, _, operator): (Ident, Ident, Punct),
    left_res: Type,
    right_res: Type,
) -> Option<TokenStream> {
    let (mut result, op_impl) = match (
        left_res.category,
        left_res.ty,
        right_res.category,
        right_res.ty,
    ) {
        (_, "bool", _, _)
        | (_, _, _, "bool")
        | (Category::MATRIX, _, _, _)
        | (_, _, Category::MATRIX, _)
        | (Category::SCALAR, _, Category::SCALAR, _) => return None,

        (lc, lt, rc, rt) if lc == rc && lt == rt && left_res.size == right_res.size => (
            left_res.name.clone(),
            match lc {
                Category::MATRIX => unreachable!(),
                Category::SCALAR => quote! {
                    self #operator rhs
                },
                Category::VECTOR => {
                    let result = left_res.name.clone();
                    let l_fields: Vec<_> = {
                        (0..left_res.size.unwrap())
                            .map(|i| Ident::new(&format!("l_{}", i), Span::call_site()))
                            .collect()
                    };
                    let r_fields: Vec<_> = {
                        (0..left_res.size.unwrap())
                            .map(|i| Ident::new(&format!("r_{}", i), Span::call_site()))
                            .collect()
                    };
                    let res_fields: Vec<_> = {
                        l_fields
                            .iter()
                            .zip(r_fields.iter())
                            .map(|(l_f, r_f)| {
                                quote! { #l_f #operator #r_f }
                            })
                            .collect()
                    };

                    quote! {
                        let #result([ #( #l_fields ),* ]) = self;
                        let #result([ #( #r_fields ),* ]) = rhs;
                        #result([ #( #res_fields ),* ])
                    }
                }
            },
        ),

        _ => return None,
    };

    let left_type = left_res.name.clone();
    let right_type = right_res.name.clone();
    let method = Ident::new(&trait_id.to_string().to_lowercase(), Span::call_site());

    if left_type.to_string() == result.to_string() {
        result = Ident::new("Self", Span::call_site());
    }

    Some(quote! {
        impl #trait_id<#right_type> for #left_type {
            type Output = #result;

            #[inline]
            fn #method(self, rhs: #right_type) -> Self::Output {
                #op_impl
            }
        }
    })
}

const MATH_OPS: [(&str, &str, char); 5] = [
    ("Add", "Add", '+'),
    ("Sub", "Subtract", '-'),
    ("Mul", "Multiply", '*'),
    ("Div", "Divide", '/'),
    ("Rem", "Modulus", '%'),
];

pub fn impl_math() -> Vec<(TokenStream, TokenStream, TokenStream, Vec<TokenStream>)> {
    MATH_OPS
        .into_iter()
        .map(|&(trait_name, node, operator)| {
            let lower = Ident::new(&trait_name.to_string().to_lowercase(), Span::call_site());
            let trait_id = Ident::new(trait_name, Span::call_site());
            let node_id = Ident::new(node, Span::call_site());
            let op = Punct::new(operator, Spacing::Alone);

            (
                quote! {
                    fn #lower<R: Copy>(lhs: Value<Self, T>, rhs: Value<Self, R>) -> Value<Self, T::Output>
                    where
                        T: #trait_id<R>,
                        T::Output: Copy,
                        Self: Container<R> + Container<T::Output>;
                },
                quote! {
                    fn #lower<R: Copy>(lhs: Value<Self, T>, rhs: Value<Self, R>) -> Value<Self, T::Output>
                    where
                        T: #trait_id<R>,
                        T::Output: Copy,
                    {
                        with_graph(|graph| {
                            let node = graph.add_node(Node::#node_id);
                            graph.add_edge(lhs.0, node, 0);
                            graph.add_edge(rhs.0, node, 1);
                            Value(node)
                        })
                    }
                },
                quote! {
                    fn #lower<R: Copy>(lhs: Value<Self, T>, rhs: Value<Self, R>) -> Value<Self, T::Output>
                    where
                        T: #trait_id<R>,
                        T::Output: Copy,
                    {
                        Value(lhs.0 #op rhs.0)
                    }
                },

                all_types()
                    .into_iter()
                    .flat_map(|left_type| {
                        all_types()
                            .into_iter()
                            .filter_map(|right_type| {
                                if trait_name == "Mul" {
                                    impl_mul_variant(
                                        left_type.clone(),
                                        right_type.clone(),
                                    )
                                } else {
                                    impl_math_variant(
                                        (
                                            Ident::new(&trait_name, Span::call_site()),
                                            Ident::new(&node, Span::call_site()),
                                            Punct::new(operator, Spacing::Alone),
                                        ),
                                        left_type.clone(),
                                        right_type.clone(),
                                    )
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect()
            )
        })
        .collect()
}
