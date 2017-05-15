//! Mul trait implementation

use quote::{Ident, Tokens};
use defs::{Category, Node, all_nodes};
use mul::impl_mul_variant;

pub fn construct_type(ty: Node) -> Tokens {
    let Node { name, args, .. } = ty;
    match args {
        Some(list) => {
            let value = list.into_iter()
                .map(|ty| ty.name);

            quote! {
                #name < #( #value ),* >
            }
        },

        None => quote! { #name },
    }
}

fn impl_math_variant((trait_id, node_id, operator): (Ident, Ident, Ident), left_type: Node, right_type: Node) -> Option<Tokens> {
    let left_res = left_type.result.clone();
    let right_res = right_type.result.clone();

    let (result, op_impl) = match (left_res.category, left_res.ty, right_res.category, right_res.ty) {
        (_, "bool", _, _) |
        (_, _, _, "bool") |
        (Category::MATRIX, _, _, _) |
        (_, _, Category::MATRIX, _) |
        (Category::SCALAR, _, Category::SCALAR, _) => return None,

        (lc, lt, rc, rt) if lc == rc && lt == rt && left_res.size == right_res.size => (
            left_res.name.clone(),
            match lc {
                Category::MATRIX => unreachable!(),
                Category::SCALAR => quote! {
                    return (left_val #operator right_val).into();
                },
                Category::VECTOR => {
                    let result = left_res.name.clone();
                    let l_fields: Vec<_> = {
                        (0..left_res.size.unwrap())
                            .map(|i| Ident::from(format!("l_{}", i)))
                            .collect()
                    };
                    let r_fields: Vec<_> = {
                        (0..left_res.size.unwrap())
                            .map(|i| Ident::from(format!("r_{}", i)))
                            .collect()
                    };
                    let res_fields: Vec<_> = {
                        l_fields.iter()
                            .zip(r_fields.iter())
                            .map(|(l_f, r_f)| {
                                quote! { #l_f #operator #r_f }
                            })
                            .collect()
                    };

                    quote! {
                        let #result( #( #l_fields ),* ) = left_val;
                        let #result( #( #r_fields ),* ) = right_val;
                        return #result( #( #res_fields ),* ).into();
                    }
                },
            }
        ),

        _ => return None,
    };

    let left_type = construct_type(left_type);
    let right_type = construct_type(right_type);
    let method = Ident::from(trait_id.to_string().to_lowercase());

    let tokens = quote! {
        impl #trait_id<#right_type> for #left_type {
            type Output = Value<#result>;

            #[inline]
            fn #method(self, rhs: #right_type) -> Self::Output {
                if let (Some(left_val), Some(right_val)) = (self.get_concrete(), rhs.get_concrete()) {
                    #op_impl
                }

                let graph_opt = self.get_graph().or(rhs.get_graph());
                if let Some(graph_ref) = graph_opt {
                    let left_src = self.get_index(graph_ref.clone());
                    let right_src = rhs.get_index(graph_ref.clone());

                    let index = {
                        let mut graph = graph_ref.borrow_mut();
                        let index = graph.add_node(Node::#node_id);
                        graph.add_edge(left_src, index, 0);
                        graph.add_edge(right_src, index, 1);
                        index
                    };

                    return Value::Abstract {
                        graph: graph_ref.clone(),
                        index,
                        ty: PhantomData,
                    };
                }

                unreachable!()
            }
        }
    };

    Some(tokens)
}

const MATH_OPS: [(&str, &str, &str); 4] = [
    ("Add", "Add", "+"),
    ("Sub", "Subtract", "-"),
    ("Div", "Divide", "/"),
    ("Rem", "Modulus", "%"),
];

pub fn impl_math() -> Vec<Tokens> {
    all_nodes().into_iter()
        .flat_map(|left_type| {
            all_nodes().into_iter()
                .flat_map(|right_type| {
                    MATH_OPS.into_iter()
                        .filter_map(|&(trait_name, node, operator)| {
                            impl_math_variant(
                                (Ident::from(trait_name), Ident::from(node), Ident::from(operator)),
                                left_type.clone(),
                                right_type.clone(),
                            )
                        })
                        .chain(
                            impl_mul_variant(
                                left_type.clone(),
                                right_type.clone(),
                            )
                            .into_iter()
                        )
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
