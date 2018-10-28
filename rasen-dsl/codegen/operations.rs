//! GLSL Operation declarations

use proc_macro2::{Ident, Span, TokenStream};

fn operation(
    name: &str,
    args: u32,
    adnl_generics: &[Ident],
    constraints: &TokenStream,
    implementation: &TokenStream,
) -> TokenStream {
    let node = Ident::new(&name, Span::call_site());
    let fn_name = Ident::new(&name.to_lowercase(), Span::call_site());
    let indices: Vec<u32> = (0..args).collect();
    let mut generics: Vec<_> = {
        indices
            .iter()
            .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
            .collect()
    };
    let args: Vec<Ident> = {
        indices
            .iter()
            .map(|i| Ident::new(&format!("arg_{}", i), Span::call_site()))
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

pub fn match_values(names: &[Ident], concrete: &TokenStream, index: TokenStream) -> TokenStream {
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

                    let ident1: Vec<_> = (0..indices.len()).map(|i| Ident::new(&format!("tmp_{}", i), Span::call_site())).collect();
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

pub fn impl_operations() -> Vec<TokenStream> {
    let operations: &[(&str, u32, &[Ident], TokenStream, TokenStream)] = &[
        (
            "Normalize", 1, &[ Ident::new("S", Span::call_site()) ],
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
            "Dot", 2, &[ Ident::new("V", Span::call_site()) ],
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
            "Cross", 2, &[ Ident::new("S", Span::call_site()) ],
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
            "Length", 1, &[ Ident::new("V", Span::call_site()) ],
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
            "Distance", 2, &[ Ident::new("V", Span::call_site()) ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R> + Sub<V, Output=V>, R: Floating },
            quote! {
                length(arg_0 - arg_1)
            }
        ), (
            "Reflect", 2, &[ Ident::new("S", Span::call_site()) ],
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
            "Refract", 3, &[ Ident::new("S", Span::call_site()) ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=S>, R: Math + Vector<S> + Sub<R, Output=R>, S: Floating + Mul<R, Output=R> },
            quote! {
                let one: S = S::one();
                let dot: S = match dot(arg_1, arg_0) {
                    Value::Concrete(v) => v,
                    _ => unreachable!(),
                };
                let k: S = one - arg_2 * arg_2 * (one - dot * dot);

                let res: R = if k < S::zero() {
                    R::zero()
                } else {
                    let lhs = arg_2 * arg_0;
                    let rhs = (arg_2 * dot + k.sqrt()) * arg_1;
                    lhs - rhs
                };

                res.into()
            }
        ), (
            "Mix", 3, &[ Ident::new("V0", Span::call_site()) ],
            quote!{
                where T0: IntoValue<Output=V0>,
                    T1: IntoValue<Output=V0>,
                    T2: IntoValue<Output=V0>,
                    V0: Math + Into<Value<R>> + Clone,
                    Value<V0>: IntoValue
            },
            quote! {
                let a: V0 = V0::one() - arg_2.clone();
                let b: V0 = arg_0 * a;
                let c: V0 = arg_1 * arg_2;
                let d: V0 = b + c;
                let r: Value<R> = d.into();
                r
            }
        ), (
            "Sqrt", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.sqrt().into()
            }
        ), (
            "Log", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.ln().into()
            }
        ), (
            "Abs", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Floating },
            quote! {
                arg_0.abs().into()
            }
        ), (
            "Smoothstep", 3, &[],
            quote! {
                where T0: IntoValue<Output=R>,
                    T1: IntoValue<Output=R>,
                    T2: IntoValue<Output=R>,
                    R: Floating + Mul<Value<R>, Output=Value<R>> + Sub<Value<R>, Output=Value<R>>,
                    Value<R>: Mul</*Value<R>,*/ Output=Value<R>> + Clone
            },
            quote! {
                let t: Value<R> = clamp(
                    (arg_2 - arg_0) / (arg_1 - arg_0),
                    R::zero(), R::one(),
                );

                let a: Value<R> = t.clone() * t.clone();
                let b: Value<R> = R::two() * t;
                let c: Value<R> = R::three() - b;

                a * c
            }
        ), (
            "Inverse", 1, &[ Ident::new("V", Span::call_site()), Ident::new("S", Span::call_site()) ],
            quote! { where T0: IntoValue<Output=R>, R: Matrix<V, S>, V: Vector<S>, S: Scalar },
            quote! {
                unimplemented!()
            }
        ),  (
            "Step", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar },
            quote! {
                if arg_1 < arg_0 {
                    R::zero().into()
                } else {
                    R::one().into()
                }
            }
        ),
    ];

    operations
        .into_iter()
        .map(
            |&(name, args, generics, ref constraints, ref implementation)| {
                operation(name, args, generics, constraints, implementation)
            },
        )
        .collect()
}
