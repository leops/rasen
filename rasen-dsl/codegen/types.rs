//! GLSL Types declarations

use quote::{Ident, Tokens};
use codegen::defs::{Category, Type, all_types};
use codegen::operations::match_values;

fn trait_scalar(name: Ident, ty: Ident, (zero, one): (Ident, Ident)) -> Tokens {
    quote! {
        impl Scalar for #name {
            fn zero() -> Self { #zero }
            fn one() -> Self { #one }
        }

        impl ValueIter<#ty> for #name {
            type Iter = ::std::iter::Once<Value<#ty>>;
            fn iter<'a>(obj: &Self) -> Self::Iter {
                ::std::iter::once(obj.into())
            }
        }
    }
}

fn trait_numerical(name: Ident, pow_fn: Ident, pow_ty: Ident) -> Tokens {
    quote! {
        impl Numerical for #name {
            fn pow(x: Self, y: Self) -> Self { x.#pow_fn(y as #pow_ty) }
        }
    }
}

fn trait_integer(name: Ident, is_signed: bool) -> Tokens {
    quote! {
        impl Integer for #name {
            fn is_signed() -> bool { #is_signed }
        }
    }
}

fn trait_float(name: Ident, is_double: bool) -> Tokens {
    quote! {
        impl Floating for #name {
            fn is_double() -> bool { #is_double }
            fn sqrt(self) -> Self { self.sqrt() }
            fn floor(self) -> Self { self.floor() }
            fn ceil(self) -> Self { self.ceil() }
            fn round(self) -> Self { self.round() }
            fn sin(self) -> Self { self.sin() }
            fn cos(self) -> Self { self.cos() }
            fn tan(self) -> Self { self.tan() }
        }
    }
}

fn trait_vector(name: Ident, component_count: u32, component_type: Ident, (zero, one): (Ident, Ident)) -> Tokens {
    let fields_zero: Vec<_> = (0..component_count).map(|_| zero.clone()).collect();
    let fields_one: Vec<_> = (0..component_count).map(|_| one.clone()).collect();
    let obj_fields: Vec<_> = {
        (0..component_count)
            .map(|i| {
                let idx = Ident::from(format!("{}", i));
                quote! { obj.#idx.into() }
            })
            .collect()
    };

    quote! {
        impl Vector<#component_type> for #name {
            fn zero() -> Self { #name( #( #fields_zero ),* ) }
            fn one() -> Self { #name( #( #fields_one ),* ) }
            fn component_count() -> u32 { #component_count }
        }

        impl ValueIter<#component_type> for #name {
            type Iter = ::std::vec::IntoIter<Value<#component_type>>;
            fn iter<'a>(obj: &Self) -> Self::Iter {
                vec![ #( #obj_fields ),* ].into_iter()
            }
        }
    }
}

fn trait_matrix(name: Ident, column_count: u32, column_type: Ident, scalar_type: Ident, (zero, one): (Ident, Ident)) -> Tokens {
    let identity: Vec<_> = {
        (0..column_count)
            .flat_map(|x| {
                (0..column_count)
                    .map(|y| if x == y { one.clone() } else { zero.clone() })
                    .collect::<Vec<_>>()
            })
            .collect()
    };

    quote! {
        impl Matrix<#column_type, #scalar_type> for #name {
            fn identity() -> Self { #name( #identity ) }
            fn column_count() -> u32 { #column_count }
        }

        impl ValueIter<#scalar_type> for #name {
            type Iter = ::std::vec::IntoIter<Value<#scalar_type>>;
            fn iter<'a>(obj: &Self) -> Self::Iter {
                let lst: Vec<_> = obj.0.into_iter().map(|s| s.into()).collect();
                lst.into_iter()
            }
        }
    }
}

fn type_values(ty: &str) -> (Ident, Ident) {
    match ty {
        "bool" => {
            (Ident::from("false"), Ident::from("true"))
        },

        "i32" | "u32" => {
            (Ident::from("0"), Ident::from("1"))
        },

        "f32" | "f64" => {
            (Ident::from("0.0"), Ident::from("1.0"))
        },

        _ => unreachable!(),
    }
}

fn type_scalar(name: &Ident, ty: &'static str) -> Tokens {
    let mut traits = vec![
        trait_scalar(name.clone(), Ident::from(ty), type_values(ty)),
    ];

    match ty {
        "bool" => {},

        "i32" | "u32" => {
            traits.push(trait_numerical(name.clone(), Ident::from("pow"), Ident::from("u32")));
            traits.push(trait_integer(name.clone(), ty == "i32"));
        },

        "f32" | "f64" => {
            traits.push(trait_numerical(name.clone(), Ident::from("powf"), Ident::from(ty)));
            traits.push(trait_float(name.clone(), ty == "f64"));
        },

        _ => unreachable!(),
    }

    let ty = Ident::from(ty);
    quote! {
        pub type #name = #ty;
        #( #traits )*

        impl Into<Value<#name>> for #name {
            fn into(self) -> Value<#name> {
                Value::Concrete(self)
            }
        }
        impl<'a> Into<Value<#name>> for &'a #name {
            fn into(self) -> Value<#name> {
                Value::Concrete(*self)
            }
        }

        impl IntoValue for #name {
            type Output = #name;

            fn into_value(self) -> Value<#name> {
                Value::Concrete(self)
            }

            fn get_index(&self, mut graph: GraphRef) -> NodeIndex<u32> {
                graph.add_node(Node::Constant(TypedValue::#name(*self)))
            }
        }
    }
}

fn type_vector(name: &Ident, ty: &'static str, component: Option<Box<Type>>, size: Option<u32>) -> Tokens {
    let component = component.unwrap();
    let size = size.unwrap();

    let traits = vec![
        trait_vector(name.clone(), size, component.name.clone(), type_values(ty))
    ];

    let ty = Ident::from(ty);
    let types = (0..size).map(|_| ty.clone());

    let fields = (0..size).map(|i| {
        let id = Ident::from(format!("{}", i));
        quote! { self.#id }
    });

    let into_idx: Vec<_> = {
        (0..size)
        .map(|i| {
            let index = i as usize;
            quote! { arr[#index] }
        })
        .collect()
    };

    let index_arms = (0..size).map(|i| {
        let index = Ident::from(format!("{}", i));
        quote! { #i => &self.#index }
    });

    let lower = Ident::from(name.as_ref().to_lowercase());
    let upper = Ident::from(name.as_ref().to_uppercase());
    let indices: Vec<u32> = (0..size).collect();
    let generics: Vec<_> = (0..size).map(|i| Ident::from(format!("T{}", i))).collect();
    let args1: Vec<_> = generics.iter().map(|gen| Ident::from(gen.as_ref().to_lowercase())).collect();
    let args2 = args1.clone();
    let args3 = args1.clone();
    let arg_list: Vec<_> = {
        generics.iter()
            .zip(args1.iter())
            .map(|(gen, name)| quote! { #name: #gen })
            .collect()
    };
    let constraints: Vec<_> = {
        generics.iter()
            .map(|gen| {
                let comp = component.name.clone();
                quote! { #gen: IntoValue<Output=#comp> }
            })
            .collect()
    };

    let func_impl = match_values(
        &args1,
        &quote! {
            Value::Concrete(#name( #( #args3 ),* ))
        },
        quote! {
            let index = graph.add_node(Node::Construct(TypeName::#upper));
            #( graph.add_edge(#args2, index, #indices); )*
            index
        },
    );

    quote! {
        #[derive(Copy, Clone, Debug)]
        pub struct #name( #( pub #types ),* );
        #( #traits )*

        impl From<Vec<#ty>> for #name {
            fn from(arr: Vec<#ty>) -> #name {
                #name( #( #into_idx ),* )
            }
        }

        impl Index<u32> for #name {
            type Output = #ty;
            fn index(&self, index: u32) -> &Self::Output {
                match index {
                    #( #index_arms, )*
                    _ => unreachable!(),
                }
            }
        }

        impl Into<Value<#name>> for #name {
            fn into(self) -> Value<#name> {
                Value::Concrete(self)
            }
        }

        impl IntoValue for #name {
            type Output = #name;

            fn into_value(self) -> Value<#name> {
                Value::Concrete(self)
            }

            fn get_index(&self, mut graph: GraphRef) -> NodeIndex<u32> {
                graph.add_node(Node::Constant(TypedValue::#name( #( #fields ),* )))
            }
        }

        #[inline]
        pub fn #lower< #( #generics ),* >( #( #arg_list ),* ) -> Value<#name> where #( #constraints ),* {
            #func_impl
        }
    }
}

fn type_matrix(name: &Ident, ty: &'static str, component: Option<Box<Type>>, size: Option<u32>) -> Tokens {
    let vector = component.unwrap();
    let scalar = vector.clone().component.unwrap();
    let size = size.unwrap();

    let traits = vec![
        trait_matrix(name.clone(), size, vector.name.clone(), scalar.name.clone(), type_values(ty))
    ];

    let ty = Ident::from(ty);
    let mat_size = (size * size) as usize;
    
    let lower = Ident::from(name.as_ref().to_lowercase());
    let upper = Ident::from(name.as_ref().to_uppercase());
    let indices: Vec<u32> = (0..size).collect();
    let generics: Vec<_> = (0..size).map(|i| Ident::from(format!("T{}", i))).collect();
    let args1: Vec<_> = generics.iter().map(|gen| Ident::from(gen.as_ref().to_lowercase())).collect();
    let args2 = args1.clone();
    let args3: Vec<_> = {
        args1.iter()
            .flat_map(|ident| -> Vec<_> {
                (0..size)
                    .map(|index| {
                        let index = Ident::from(format!("{}", index));
                        quote! { #ident.#index }
                    })
                    .collect()
            })
            .collect()
    };
    let arg_list: Vec<_> = {
        generics.iter()
            .zip(args1.iter())
            .map(|(gen, name)| quote! { #name: #gen })
            .collect()
    };
    let constraints: Vec<_> = {
        generics.iter()
            .map(|gen| {
                let comp = vector.name.clone();
                quote! { #gen: IntoValue<Output=#comp> }
            })
            .collect()
    };

    let func_impl = match_values(
        &args1,
        &quote! {
            Value::Concrete(#name([ #( #args3 ),* ]))
        },
        quote! {
            let index = graph.add_node(Node::Construct(TypeName::#upper));
            #( graph.add_edge(#args2, index, #indices); )*
            index
        },
    );

    quote! {
        #[derive(Copy, Clone, Debug)]
        pub struct #name(pub [ #ty ; #mat_size ]);
        #( #traits )*

        impl Into<Value<#name>> for [ #ty ; #mat_size ] {
            fn into(self) -> Value<#name> {
                Value::Concrete(#name(self))
            }
        }

        impl Into<Value<#name>> for #name {
            fn into(self) -> Value<#name> {
                Value::Concrete(self)
            }
        }

        impl Index<u32> for #name {
            type Output = #ty;
            fn index(&self, index: u32) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        impl IntoValue for #name {
            type Output = #name;

            fn into_value(self) -> Value<#name> {
                Value::Concrete(self)
            }

            fn get_index(&self, mut graph: GraphRef) -> NodeIndex<u32> {
                graph.add_node(Node::Constant(TypedValue::#name(self.0)))
            }
        }

        #[inline]
        pub fn #lower< #( #generics ),* >( #( #arg_list ),* ) -> Value<#name> where #( #constraints ),* {
            #func_impl
        }
    }
}

pub fn type_structs() -> Vec<Tokens> {
    all_types().iter()
        .filter_map(|ty| {
            let Type { name, category, ty, component, size, .. } = ty.clone();
            let decl = match category {
                Category::SCALAR => type_scalar(&name, ty),
                Category::VECTOR => type_vector(&name, ty, component, size),
                Category::MATRIX => type_matrix(&name, ty, component, size),
            };

            let upper = Ident::from(
                name.as_ref().to_string().to_uppercase()
            );

            Some(quote! {
                #decl

                impl Input<#name> for Module {
                    #[inline]
                    fn input<N>(&self, location: u32, name: N) -> Value<#name> where N: Into<NameWrapper> {
                        let index = {
                            let mut module = self.borrow_mut();
                            let NameWrapper(name) = name.into();
                            module.main.add_node(Node::Input(location, TypeName::#upper, name))
                        };

                        Value::Abstract {
                            module: self.clone(),
                            function: FuncKind::Main,
                            index,
                            ty: PhantomData,
                        }
                    }
                }

                impl Uniform<#name> for Module {
                    #[inline]
                    fn uniform<N>(&self, location: u32, name: N) -> Value<#name> where N: Into<NameWrapper> {
                        let index = {
                            let mut module = self.borrow_mut();
                            let NameWrapper(name) = name.into();
                            module.main.add_node(Node::Uniform(location, TypeName::#upper, name))
                        };

                        Value::Abstract {
                            module: self.clone(),
                            function: FuncKind::Main,
                            index,
                            ty: PhantomData,
                        }
                    }
                }

                impl Output<#name> for Module {
                    #[inline]
                    fn output<N>(&self, location: u32, name: N, source: Value<#name>) where N: Into<NameWrapper> {
                        let src = match source {
                            Value::Abstract { index, .. } => index,
                            source @ Value::Concrete(_) => {
                                let module = self.borrow_mut();
                                let graph = FuncKind::Main.get_graph_mut(module);
                                source.get_index(graph)
                            },
                        };

                        let mut module = self.borrow_mut();
                        let NameWrapper(name) = name.into();
                        let sink = module.main.add_node(Node::Output(location, TypeName::#upper, name));
                        module.main.add_edge(src, sink, 0);
                    }
                }

                impl<F> Parameter<#name> for Function<F> {
                    #[inline]
                    fn parameter(&self, location: u32) -> Value<#name> {
                        let index = {
                            let mut module = self.module.borrow_mut();
                            let graph = &mut module[self.func];
                            graph.add_node(Node::Parameter(location, TypeName::#upper))
                        };

                        Value::Abstract {
                            module: self.module.clone(),
                            function: FuncKind::Ref(self.func),
                            index,
                            ty: PhantomData,
                        }
                    }
                }
            })
        })
        .collect()
}
