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
    
    let args2 = args.clone();

    for gen in adnl_generics {
        generics.push(gen.clone());
    }

    let method_impl = match_values(
        &args,
        implementation,
        quote! {
            let index = graph.add_node(Node::#node);
            #( graph.add_edge(#args2, index, #indices); )*
            index
        },
    );

    quote! {
        #[allow(unused_variables)]
        pub fn #fn_name< #( #generics , )* R >( #( #arg_list ),* ) -> Value<R> #constraints {
            #method_impl
        }
    }
}

pub fn match_values(names: &[Ident], concrete: &Tokens, index: Tokens) -> Tokens {
    let parts = names.len();
    let arms: Vec<_> = {
        (0..2u32.pow(parts as u32))
            .map(|id| {
                if id == 0 {
                    quote! {
                        ( #( Value::Concrete( #names ), )* ) => {
                            return { #concrete }
                        },
                    }
                } else {
                    let mut first_abstract = true;
                    let (patterns, indices): (Vec<_>, Vec<_>) = {
                        names.into_iter().enumerate()
                            .map(move |(i, name)| {
                                if (id >> i) & 1 == 1 {
                                    (
                                        if first_abstract {
                                            first_abstract = false;
                                            quote! { Value::Abstract { module, function, index: #name, .. } }
                                        } else {
                                            quote! { Value::Abstract { index: #name, .. } }
                                        },
                                        quote! { #name }
                                    )
                                } else {
                                    (
                                        quote! { #name @ Value::Concrete(_) },
                                        quote! {{
                                            let module = module.borrow_mut();
                                            let graph = function.get_graph_mut(module);
                                            #name.get_index(graph)
                                        }}
                                    )
                                }
                            })
                            .unzip()
                    };

                    let ident1: Vec<_> = (0..indices.len()).map(|i| Ident::from(format!("tmp_{}", i))).collect();
                    let ident2 = ident1.clone();

                    quote! {
                        ( #( #patterns, )* ) => {
                            #( let #ident1 = #indices; )*
                            ( module, function, #( #ident2 ),* )
                        },
                    }
                }
            })
            .collect()
    };

    quote! {
        let ( module, function, #( #names ),* ) = match ( #( #names.into_value(), )* ) {
            #( #arms )*
        };

        let index = {
            let module = module.borrow_mut();
            let mut graph = function.get_graph_mut(module);
            #index
        };

        Value::Abstract {
            module,
            function,
            index,
            ty: PhantomData,
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
                let length = match length(arg_0) {
                    Value::Concrete(v) => v,
                    _ => unreachable!(),
                };
                let arr: Vec<_> = (0..count).map(|i| arg_0[i] / length).collect();
                let vec: R = arr.into();
                vec.into()
            }
        ), (
            "Dot", 2, &[ Ident::from("V") ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R>, R: Numerical },
            quote! {
                let count = V::component_count();
                let val: R = (0..count).map(|i| arg_0[i] * arg_1[i]).sum();
                val.into()
            }
        ), (
            "Clamp", 3, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=R>, R: Scalar },
            quote! {
                let x: Value<R> = arg_0.into();
                let min_val: Value<R> = arg_1.into();
                let max_val: Value<R> = arg_2.into();
                min(max(x, min_val), max_val)
            }
        ), (
            "Cross", 2, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S>, S: Numerical },
            quote! {
                let vec: R = vec![
                    arg_0[1] * arg_1[2] - arg_1[1] * arg_0[2],
                    arg_0[2] * arg_1[0] - arg_1[2] * arg_0[0],
                    arg_0[0] * arg_1[1] - arg_1[0] * arg_0[1],
                ].into();
                vec.into()
            }
        ), (
            "Floor", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.floor().into()
            }
        ), (
            "Ceil", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.ceil().into()
            }
        ), (
            "Round", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.round().into()
            }
        ), (
            "Sin", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.sin().into()
            }
        ), (
            "Cos", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.cos().into()
            }
        ), (
            "Tan", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.tan().into()
            }
        ), (
            "Pow", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Numerical },
            quote! {
                Numerical::pow(arg_0, arg_1).into()
            }
        ), (
            "Min", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar },
            quote! {
                (if arg_1 < arg_0 { arg_1 } else { arg_0 }).into()
            }
        ), (
            "Max", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar },
            quote! {
                (if arg_1 > arg_0 { arg_1 } else { arg_0 }).into()
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

                length.sqrt().into()
            }
        ), (
            "Distance", 2, &[ Ident::from("V") ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R> + Sub<V, Output=V>, R: Floating },
            quote! {
                length(arg_0 - arg_1)
            }
        ), (
            "Reflect", 2, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S> + Sub<R, Output=R>, S: Numerical + Mul<R, Output=R> },
            quote! {
                let res: S = match dot(arg_1, arg_0) {
                    Value::Concrete(v) => v,
                    _ => unreachable!(),
                };
                let res: S = res + res;
                let res: R = arg_0 - res * arg_1;
                res.into()
            }
        ), (
            "Refract", 3, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=S>, R: Vector<S> + Sub<R, Output=R>, S: Floating + Mul<R, Output=R> },
            quote! {
                let one: S = Scalar::one();
                let dot: S = match dot(arg_1, arg_0) {
                    Value::Concrete(v) => v,
                    _ => unreachable!(),
                };
                let k: S = one - arg_2 * arg_2 * (one - dot * dot);

                let res: R = if k < Scalar::zero() {
                    Vector::zero()
                } else {
                    let lhs = arg_2 * arg_0;
                    let rhs = (arg_2 * dot + k.sqrt()) * arg_1;
                    lhs - rhs
                };

                res.into()
            }
        ), (
            "Mix", 3, &[ Ident::from("V0"), Ident::from("V1"), Ident::from("V2"), Ident::from("V3"), Ident::from("V4") ],
            quote! { where T0: IntoValue<Output=V0>, T1: IntoValue<Output=V1>, T2: IntoValue<Output=V2>, V0: IntoValue + Copy + Add<V4, Output=R>, V1: IntoValue + Clone + Sub<V0, Output=V3>, V2: IntoValue + Clone + Mul<V3, Output=V4>, R: Into<Value<R>> },
            quote! {
                (arg_0 + arg_2 * (arg_1 - arg_0)).into()
            }
        )
    ];

    operations.into_iter()
        .map(|&(name, args, generics, ref constraints, ref implementation)| {
            operation(name, args, generics, constraints, implementation)
        })
        .collect()
}
