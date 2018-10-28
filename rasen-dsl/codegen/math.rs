//! Mul trait implementation

use codegen::{
    defs::{all_nodes, Category, Node},
    mul::impl_mul_variant,
    operations::match_values,
};
use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};

pub fn construct_type(ty: Node) -> TokenStream {
    let Node { name, args, .. } = ty;
    match args {
        Some(list) => {
            let value = list.into_iter().map(|ty| ty.name);

            quote! {
                #name < #( #value ),* >
            }
        }

        None => quote! { #name },
    }
}

#[allow(clippy::match_same_arms)]
fn impl_math_variant(
    (trait_id, node_id, operator): (Ident, Ident, Punct),
    left_type: Node,
    right_type: Node,
) -> Option<TokenStream> {
    let left_res = left_type.result.clone();
    let right_res = right_type.result.clone();

    let is_raw = !left_type.is_value() && !right_type.is_value();
    let (mut result, op_impl) = match (
        left_res.category,
        left_res.ty,
        right_res.category,
        right_res.ty,
    ) {
        (_, "bool", _, _)
        | (_, _, _, "bool")
        | (Category::MATRIX, _, _, _)
        | (_, _, Category::MATRIX, _) => return None,

        (Category::SCALAR, _, Category::SCALAR, _) if is_raw => return None,

        (lc, lt, rc, rt) if lc == rc && lt == rt && left_res.size == right_res.size => (
            left_res.name.clone(),
            match lc {
                Category::MATRIX => unreachable!(),
                Category::SCALAR => quote! {
                    (lhs #operator rhs).into()
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
                        let #result( #( #l_fields ),* ) = lhs;
                        let #result( #( #r_fields ),* ) = rhs;
                        #result( #( #res_fields ),* ).into()
                    }
                }
            },
        ),

        _ => return None,
    };

    let left_type = construct_type(left_type);
    let right_type = construct_type(right_type);
    let method = Ident::new(&trait_id.to_string().to_lowercase(), Span::call_site());

    if left_type.to_string() == result.to_string() {
        result = Ident::new("Self", Span::call_site());
    }

    if is_raw {
        Some(quote! {
            impl #trait_id<#right_type> for #left_type {
                type Output = #result;

                #[inline]
                fn #method(self, rhs: #right_type) -> Self::Output {
                    let lhs = self;
                    #op_impl
                }
            }
        })
    } else {
        let method_impl = match_values(
            &[
                Ident::new("lhs", Span::call_site()),
                Ident::new("rhs", Span::call_site()),
            ],
            &op_impl,
            quote! {
                let index = graph.add_node(Node::#node_id);
                graph.add_edge(lhs, index, 0);
                graph.add_edge(rhs, index, 1);
                index
            },
        );

        let tokens = quote! {
            impl #trait_id<#right_type> for #left_type {
                type Output = Value<#result>;

                #[inline]
                fn #method(self, rhs: #right_type) -> Self::Output {
                    let lhs = self;
                    #method_impl
                }
            }
        };

        Some(tokens)
    }
}

const MATH_OPS: [(&str, &str, char); 4] = [
    ("Add", "Add", '+'),
    ("Sub", "Subtract", '-'),
    ("Div", "Divide", '/'),
    ("Rem", "Modulus", '%'),
];

pub fn impl_math() -> Vec<TokenStream> {
    all_nodes()
        .into_iter()
        .flat_map(|left_type| {
            all_nodes()
                .into_iter()
                .flat_map(|right_type| {
                    MATH_OPS
                        .into_iter()
                        .filter_map(|&(trait_name, node, operator)| {
                            impl_math_variant(
                                (
                                    Ident::new(&trait_name, Span::call_site()),
                                    Ident::new(&node, Span::call_site()),
                                    Punct::new(operator, Spacing::Alone),
                                ),
                                left_type.clone(),
                                right_type.clone(),
                            )
                        })
                        .chain(impl_mul_variant(left_type.clone(), right_type.clone()).into_iter())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
