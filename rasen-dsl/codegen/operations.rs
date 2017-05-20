//! GLSL Operation declarations

use quote::{Ident, Tokens};

fn operation(name: &str, args: u32, adnl_generics: &[Ident], constraints: &Tokens, implementation: &Tokens) -> Tokens {
    let node = Ident::from(name);
    let fn_name = Ident::from(name.to_lowercase());
    let indices: Vec<u32> = (0..args).collect();
    let mut generics: Vec<_> = {
        indices.iter()
            .map(|i| Ident::from(format!("T{}", i)))
            .collect()
    };
    let args: Vec<Ident> = {
        indices.iter()
            .map(|i| Ident::from(format!("arg_{}", i)))
            .collect()
    };
    let arg_list: Vec<_> = {
        args.iter()
            .zip(generics.iter())
            .map(|(arg, ty)| quote! { #arg: #ty })
            .collect()
    };
    let srcs1: Vec<_> = {
        args.iter()
            .map(|arg| Ident::from(format!("{}_src", arg)))
            .collect()
    };
    let srcs2 = srcs1.clone();

    for gen in adnl_generics {
        generics.push(gen.clone());
    }

    let graph_opt = {
        args.iter()
            .map(|arg| quote! { #arg.get_graph() })
            .fold(None, |root, item| match root {
                Some(tokens) => Some(quote! { #tokens.or(#item) }),
                None => Some(item),
            })
    };

    let destruct_args = if args.len() > 1 {
        let args2 = args.clone();
        let args3 = args.clone();
        quote! { let ( #( Some(#args2) ),* ) = ( #( #args3.get_concrete() ),* ) }
    } else {
        quote! { let Some(arg_0) = arg_0.get_concrete() }
    };

    quote! {
        #[allow(unused_variables)]
        pub fn #fn_name< #( #generics , )* R >( #( #arg_list ),* ) -> Value<R> #constraints {
            if #destruct_args {
                #implementation
            }

            let graph_opt = #graph_opt;
            if let Some(graph_ref) = graph_opt {
                #( let #srcs1 = #args.get_index(graph_ref.clone()); )*

                let index = {
                    let mut graph = graph_ref.borrow_mut();
                    let index = graph.add_node(Node::#node);
                    #( graph.add_edge(#srcs2, index, #indices); )*
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
}

pub fn impl_operations() -> Vec<Tokens> {
    let operations: &[(&str, u32, &[Ident], Tokens, Tokens)] = &[
        (
            "Normalize", 1, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, R: Vector<S>, S: Floating },
            quote! {
                let count = R::component_count();
                let length = length(arg_0).get_concrete().unwrap();
                let arr: Vec<_> = (0..count).map(|i| arg_0[i] / length).collect();
                let vec: R = arr.into();
                return vec.into();
            }
        ), (
            "Dot", 2, &[ Ident::from("V") ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R>, R: Numerical },
            quote! {
                let count = V::component_count();
                let val: R = (0..count).map(|i| arg_0[i] * arg_1[i]).sum();
                return val.into();
            }
        ), (
            "Clamp", 3, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=R>, R: Scalar },
            quote! {
                let x: Value<R> = arg_0.into();
                let min_val: Value<R> = arg_1.into();
                let max_val: Value<R> = arg_2.into();
                return min(max(x, min_val), max_val);
            }
        ), (
            "Cross", 2, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S>, S: Numerical },
            quote! {
                let vec: R = vec![
                    arg_0[1] * arg_1[2] - arg_1[1] * arg_0[2],
                    arg_0[2] * arg_1[0] - arg_1[2] * arg_0[0],
                    arg_0[0] * arg_1[1] - arg_1[1] * arg_0[0],
                ].into();
                return vec.into();
            }
        ), (
            "Floor", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                return arg_0.floor().into();
            }
        ), (
            "Ceil", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                return arg_0.ceil().into();
            }
        ), (
            "Round", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                return arg_0.round().into();
            }
        ), (
            "Sin", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                return arg_0.sin().into();
            }
        ), (
            "Cos", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                return arg_0.cos().into();
            }
        ), (
            "Tan", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                return arg_0.tan().into();
            }
        ), (
            "Pow", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Numerical },
            quote! {
                return Numerical::pow(arg_0, arg_1).into();
            }
        ), (
            "Min", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar },
            quote! {
                return (if arg_1 < arg_0 { arg_1 } else { arg_0 }).into();
            }
        ), (
            "Max", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar },
            quote! {
                return (if arg_1 > arg_0 { arg_1 } else { arg_0 }).into();
            }
        ), (
            "Length", 1, &[ Ident::from("V") ],
            quote! { where T0: IntoValue<Output=V>, V: Vector<R>, R: Floating },
            quote! {
                let count = V::component_count();
                let length: R = {
                    (0..count)
                        .map(|i| arg_0[i] * arg_0[i])
                        .sum()
                };

                return length.sqrt().into();
            }
        ), (
            "Distance", 2, &[ Ident::from("V") ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R> + Sub<V, Output=V>, R: Floating },
            quote! {
                return length(arg_0 - arg_1);
            }
        ), (
            "Reflect", 2, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S> + Sub<R, Output=R>, S: Numerical + Mul<R, Output=R> },
            quote! {
                let res: S = dot(arg_1, arg_0).get_concrete().unwrap();
                let res: S = res + res;
                let res: R = arg_0 - res * arg_1;
                return res.into();
            }
        ), (
            "Refract", 3, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=S>, R: Vector<S> + Sub<R, Output=R>, S: Floating + Mul<R, Output=R> },
            quote! {
                let one: S = Scalar::one();
                let dot: S = dot(arg_1, arg_0).get_concrete().unwrap();
                let k: S = one - arg_2 * arg_2 * (one - dot * dot);

                let res: R = if k < Scalar::zero() {
                    Vector::zero()
                } else {
                    let lhs: R = (arg_2 * arg_0).get_concrete().unwrap();
                    let rhs: R = ((arg_2 * dot + k.sqrt()) * arg_1).get_concrete().unwrap();
                    lhs - rhs
                };

                return res.into();
            }
        ), (
            "Mix", 3, &[ Ident::from("V0"), Ident::from("V1"), Ident::from("V2"), Ident::from("V3"), Ident::from("V4") ],
            quote! { where T0: IntoValue<Output=V0>, T1: IntoValue<Output=V1>, T2: IntoValue<Output=V2>, V0: Copy + Add<V4, Output=R>, V1: Sub<V0, Output=V3>, V2: Mul<V3, Output=V4>, R: Into<Value<R>> },
            quote! {
                return (arg_0 + arg_2 * (arg_1 - arg_0)).into();
            }
        )
    ];

    operations.into_iter()
        .map(|&(name, args, generics, ref constraints, ref implementation)| {
            operation(name, args, generics, constraints, implementation)
        })
        .collect()
}