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
    let args1: Vec<Ident> = {
        indices.iter()
            .map(|i| Ident::from(format!("arg_{}", i)))
            .collect()
    };
    let args2 = args1.clone();
    let arg_list: Vec<_> = {
        args1.iter()
            .zip(generics.iter())
            .map(|(arg, ty)| quote! { #arg: #ty })
            .collect()
    };
    let srcs1: Vec<_> = {
        args1.iter()
            .map(|arg| Ident::from(format!("{}_src", arg)))
            .collect()
    };
    let srcs2 = srcs1.clone();

    for gen in adnl_generics {
        generics.push(gen.clone());
    }

    let graph_opt = {
        args1.iter()
            .map(|arg| quote! { #arg.get_graph() })
            .fold(None, |root, item| match root {
                Some(tokens) => Some(quote! { #tokens.or(#item) }),
                None => Some(item),
            })
    };

    quote! {
        pub fn #fn_name< #( #generics , )* R >( #( #arg_list ),* ) -> Value<R> #constraints {
            if #( #args1.get_concrete().is_some() )&&* {
                #implementation
            }

            let graph_opt = #graph_opt;
            if let Some(graph_ref) = graph_opt {
                #( let #srcs1 = #args2.get_index(graph_ref.clone()); )*

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
            "Normalize", 1, &[ Ident::from("S"), Ident::from("C") ],
            quote! { where T0: IntoValue<Output=R>, R: Vector<S> + Index<u32, Output=C> + From<Vec<C>> + Into<Value<R>>, S: Scalar, C: Copy + Mul<C, Output=C> + Sum + Sqrt + Div<Output=C> },
            quote! {
                let val = arg_0.get_concrete().unwrap();
                let count = val.component_count();
                let length: C = Sqrt::sqrt(
                    (0..count)
                        .map(|i| val[i] * val[i])
                        .sum()
                );

                let arr: Vec<_> = (0..count).map(|i| val[i] / length).collect();
                let vec: R = arr.into();
                return vec.into();
            }
        ), (
            "Dot", 2, &[ Ident::from("V"), Ident::from("C") ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R> + Index<u32, Output=C>, R: Scalar, C: Mul<C, Output=C> + Sum + Copy + Into<Value<R>> },
            quote! {
                let lhs = arg_0.get_concrete().unwrap();
                let rhs = arg_1.get_concrete().unwrap();
                let count = lhs.component_count();
                let val: C = (0..count).map(|i| lhs[i] * rhs[i]).sum();
                return val.into();
            }
        ), (
            "Clamp", 3, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=R>, R: IntoValue<Output=R> + Into<Value<R>> + Scalar },
            quote! {
                return min(max(arg_0, arg_1), arg_2);
            }
        ), (
            "Modulus", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Modulus"); }
        ), (
            "Cross", 2, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S>, S: Scalar },
            quote! { panic!("unimplemented Cross"); }
        ), (
            "Floor", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Floor"); }
        ), (
            "Ceil", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Ceil"); }
        ), (
            "Round", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Round"); }
        ), (
            "Sin", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Sin"); }
        ), (
            "Cos", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Cos"); }
        ), (
            "Tan", 1, &[],
            quote! { where T0: IntoValue<Output=R>, R: Scalar },
            quote! { panic!("unimplemented Tan"); }
        ), (
            "Pow", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar + Into<Value<R>> },
            quote! { panic!("unimplemented Pow"); }
        ), (
            "Min", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar + Into<Value<R>> },
            quote! {
                let x = arg_0.get_concrete().unwrap();
                let y = arg_1.get_concrete().unwrap();
                return (if y < x { y } else { x }).into();
            }
        ), (
            "Max", 2, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Scalar + Into<Value<R>> },
            quote! {
                let x = arg_0.get_concrete().unwrap();
                let y = arg_1.get_concrete().unwrap();
                return (if y > x { y } else { x }).into();
            }
        ), (
            "Length", 1, &[ Ident::from("V"), Ident::from("C") ],
            quote! { where T0: IntoValue<Output=V>, V: Vector<R> + Index<u32, Output=C>, R: Scalar, C: Copy + Mul<C, Output=C> + Sum + Sqrt + Into<Value<R>> },
            quote! {
                let val = arg_0.get_concrete().unwrap();
                let count = val.component_count();
                let length: C = Sqrt::sqrt(
                    (0..count)
                        .map(|i| val[i] * val[i])
                        .sum()
                );

                return length.into();
            }
        ), (
            "Distance", 2, &[ Ident::from("V") ],
            quote! { where T0: IntoValue<Output=V>, T1: IntoValue<Output=V>, V: Vector<R>, R: Scalar },
            quote! { panic!("unimplemented Distance"); }
        ), (
            "Reflect", 2, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, R: Vector<S>, S: Scalar },
            quote! { panic!("unimplemented Reflect"); }
        ), (
            "Refract", 3, &[ Ident::from("S") ],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=S>, R: Vector<S>, S: Scalar },
            quote! { panic!("unimplemented Refract"); }
        ), (
            "Mix", 3, &[],
            quote! { where T0: IntoValue<Output=R>, T1: IntoValue<Output=R>, T2: IntoValue<Output=R> },
            quote! { panic!("unimplemented Mix"); }
        )
    ];

    operations.into_iter()
        .map(|&(name, args, generics, ref constraints, ref implementation)| {
            operation(name, args, generics, constraints, implementation)
        })
        .collect()
}