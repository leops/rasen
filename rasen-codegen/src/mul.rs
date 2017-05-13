use quote::{Ident, Tokens};
use defs::{
    Category, Node,
    single_node, all_nodes,
};

// Mul trait implementation
fn construct_type(ty: Node) -> Tokens {
    let Node { name, args, .. } = ty;
    match args {
        Some(list) => {
            let value = list.into_iter()
                .map(|ty| ty.name)
                .collect::<Vec<_>>();

            quote! {
                #name < #( #value ),* >
            }
        },

        None => quote! { #name },
    }
}

fn impl_vector_times_scalar(result: Ident, size: u32, is_bool: bool, vector: Tokens, scalar: Tokens) -> Tokens {
    let v_fields: Vec<_> = {
        (0..size)
            .map(|i| Ident::from(format!("v_{}", i)))
            .collect()
    };
    let res_fields: Vec<_> = {
        v_fields.iter()
            .map(|f| if is_bool {
                quote! { #f && other }
            } else {
                quote! { #f * other }
            })
            .collect()
    };

    quote! {
        let #result( #( #v_fields ),* ) = #vector;
        let other = #scalar;
        return #result( #( #res_fields ),* ).into();
    }
}

fn impl_vector_times_matrix(result: Ident, size: u32, is_bool: bool, vector: Tokens, matrix: Tokens) -> Tokens {
    let v_fields: Vec<_> = {
        (0..size)
            .map(|i| Ident::from(format!("v_{}", i)))
            .collect()
    };
    let res_fields: Vec<_> = {
        (0..size)
            .map(|i| {
                let sum: Vec<_> = {
                    (0..size)
                        .map(|j| {
                            let f = Ident::from(format!("v_{}", j));
                            let index = ((i * size) + j) as usize;
                            if is_bool {
                                quote! { #f && matrix[#index] }
                            } else {
                                quote! { #f * matrix[#index] }
                            }
                        })
                        .collect()
                };

                quote! { #( #sum )+* }
            })
            .collect()
    };

    quote! {
        let #result( #( #v_fields ),* ) = #vector;
        let matrix = #matrix;
        return #result( #( #res_fields ),* ).into();
    }
}

fn impl_mul_variant(left_type: Node, right_type: Node) -> Option<Tokens> {
    let left_res = left_type.result.clone();
    let right_res = right_type.result.clone();

    let (result, mul_impl) = match (left_res.category, left_res.ty, right_res.category, right_res.ty) {
        (Category::CONCRETE, _, Category::CONCRETE, _) => return None,

        (lc, lt, rc, rt) if lc == rc && lt == rt && left_res.size == right_res.size => (
            left_res.name.clone(),
            match lc {
                Category::CONCRETE => unreachable!(),
                Category::SCALAR => if lt == "bool" {
                    quote! {
                        return (left_val.unwrap().0 && right_val.unwrap().0).into();
                    }
                } else {
                    quote! {
                        return (left_val.unwrap().0 * right_val.unwrap().0).into();
                    }
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
                            .map(|(l_f, r_f)| if lt == "bool" {
                                quote! { #l_f && #r_f }
                            } else {
                                quote! { #l_f * #r_f }
                            })
                            .collect()
                    };

                    quote! {
                        let #result( #( #l_fields ),* ) = left_val.unwrap();
                        let #result( #( #r_fields ),* ) = right_val.unwrap();
                        return #result( #( #res_fields ),* ).into();
                    }
                },
                Category::MATRIX => {
                    let result = left_res.name.clone();
                    let size = left_res.size.unwrap() as usize;
                    let res_fields: Vec<_> = {
                        (0..(size*size))
                            .map(|i| quote! { left_mat[#i] * right_mat[#i] })
                            .collect()
                    };

                    quote! {
                        let left_mat = left_val.unwrap().0;
                        let right_mat = right_val.unwrap().0;
                        return #result([ #( #res_fields ),* ]).into();
                    }
                },
            }
        ),

        (Category::VECTOR, lt, rc, rt) if lt == rt && left_res.size.unwrap() == right_res.size.unwrap_or(left_res.size.unwrap()) => (
            left_res.name.clone(),
            match rc {
                Category::VECTOR => unreachable!(),
                Category::CONCRETE | Category::SCALAR => {
                    impl_vector_times_scalar(
                        left_res.name.clone(),
                        left_res.size.unwrap(),
                        lt == "bool",
                        quote! { left_val.unwrap() },
                        quote! { right_val.unwrap().0 },
                    )
                },
                Category::MATRIX => impl_vector_times_matrix(
                    left_res.name.clone(),
                    left_res.size.unwrap(),
                    lt == "bool",
                    quote! { left_val.unwrap() },
                    quote! { right_val.unwrap().0 },
                ),
            }
        ),
        (lc, lt, Category::VECTOR, rt) if lt == rt && right_res.size.unwrap() == left_res.size.unwrap_or(right_res.size.unwrap()) => (
            right_res.name.clone(),
            match lc {
                Category::VECTOR => unreachable!(),
                Category::CONCRETE | Category::SCALAR => {
                    impl_vector_times_scalar(
                        right_res.name.clone(),
                        right_res.size.unwrap(),
                        rt == "bool",
                        quote! { right_val.unwrap() },
                        quote! { left_val.unwrap().0 },
                    )
                },
                Category::MATRIX => impl_vector_times_matrix(
                    right_res.name.clone(),
                    right_res.size.unwrap(),
                    lt == "bool",
                    quote! { right_val.unwrap() },
                    quote! { left_val.unwrap().0 },
                ),
            }
        ),

        _ => return None,
    };

    let left_type = construct_type(left_type);
    let right_type = construct_type(right_type);

    let tokens = quote! {
        impl Mul<#right_type> for #left_type {
            type Output = Value<#result>;

            #[inline]
            fn mul(self, rhs: #right_type) -> Self::Output {
                let left_val = self.get_concrete();
                let right_val = rhs.get_concrete();
                if left_val.is_some() && right_val.is_some() {
                    #mul_impl
                }

                let graph_opt = self.get_graph().or(rhs.get_graph());
                if let Some(graph_ref) = graph_opt {
                    let left_src = self.get_index(graph_ref.clone());
                    let right_src = rhs.get_index(graph_ref.clone());

                    let index = {
                        let mut graph = graph_ref.borrow_mut();
                        let index = graph.add_node(Node::Multiply);
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

pub fn impl_mul_single(lhs: &str, rhs: &str) -> Vec<Tokens> {
    single_node(lhs).iter()
        .flat_map(|left_type| {
            single_node(rhs).iter()
                .filter_map(|right_type| {
                    impl_mul_variant(
                        left_type.clone(),
                        right_type.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn impl_mul() -> Vec<Tokens> {
    all_nodes().iter()
        .flat_map(|left_type| {
            all_nodes().iter()
                .filter_map(|right_type| {
                    impl_mul_variant(
                        left_type.clone(),
                        right_type.clone(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
