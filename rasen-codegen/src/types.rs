use quote::{Ident, Tokens};
use defs::{
    Category, Type,
    all_types,
};

fn trait_scalar(name: Ident) -> Tokens {
    quote! {
        impl Scalar for #name {}
    }
}

fn trait_numerical(name: Ident) -> Tokens {
    quote! {
        impl Numerical for #name {}
    }
}

fn trait_integer(name: Ident, is_signed: bool) -> Tokens {
    quote! {
        impl Integer for #name {
            fn is_signed(&self) -> bool {
                return #is_signed;
            }
        }
    }
}

fn trait_float(name: Ident, is_double: bool) -> Tokens {
    quote! {
        impl Floating for #name {
            fn is_double(&self) -> bool {
                return #is_double;
            }
        }
    }
}

fn trait_vector(name: Ident, component_count: u32, component_type: Ident) -> Tokens {
    quote! {
        impl Vector<#component_type> for #name {
            fn component_count(&self) -> u32 {
                #component_count
            }
        }
    }
}

fn trait_matrix(name: Ident, column_count: u32, column_type: Ident, scalar_type: Ident) -> Tokens {
    quote! {
        impl Matrix<#column_type, #scalar_type> for #name {
            fn column_count(&self) -> u32 {
                #column_count
            }
        }
    }
}

pub fn type_structs() -> Vec<Tokens> {
    let res: Vec<_> = all_types().iter()
        .filter_map(|ty| {
            let Type { name, category, ty, component, size, .. } = ty.clone();
            let decl = match category {
                Category::SCALAR => {
                    let mut traits = Vec::new();

                    match ty {
                        "bool" => {
                            traits.push(trait_scalar(name.clone()));
                        },

                        "i32" | "u32" => {
                            traits.push(trait_scalar(name.clone()));
                            traits.push(trait_numerical(name.clone()));
                            traits.push(trait_integer(name.clone(), ty == "i32"));
                        },

                        "f32" | "f64" => {
                            traits.push(trait_scalar(name.clone()));
                            traits.push(trait_numerical(name.clone()));
                            traits.push(trait_float(name.clone(), ty == "f64"));
                        },

                        _ => unreachable!(),
                    }

                    let ty = Ident::from(ty);
                    quote! {
                        #[derive(Copy, Clone, PartialEq, PartialOrd)]
                        pub struct #name(pub #ty);
                        #( #traits )*

                        impl Into<Value<#name>> for #ty {
                            fn into(self) -> Value<#name> {
                                Value::Concrete(#name(self))
                            }
                        }
                        impl<'a> Into<Value<#name>> for &'a #ty {
                            fn into(self) -> Value<#name> {
                                Value::Concrete(#name(*self))
                            }
                        }

                        impl IntoValue for #ty {
                            type Output = #name;

                            fn get_graph(&self) -> Option<GraphRef> {
                                None
                            }

                            fn get_concrete(&self) -> Option<Self::Output> {
                                Some(#name(*self))
                            }

                            fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
                                let mut graph = graph.borrow_mut();
                                graph.add_node(Node::Constant(TypedValue::#name(*self)))
                            }
                        }

                        impl Into<Value<#name>> for #name {
                            fn into(self) -> Value<#name> {
                                Value::Concrete(self)
                            }
                        }

                        impl IntoValue for #name {
                            type Output = #name;

                            fn get_graph(&self) -> Option<GraphRef> {
                                None
                            }

                            fn get_concrete(&self) -> Option<Self::Output> {
                                Some(*self)
                            }

                            fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
                                let mut graph = graph.borrow_mut();
                                graph.add_node(Node::Constant(TypedValue::#name(self.0)))
                            }
                        }
                    }
                },

                Category::VECTOR => {
                    let component = component.unwrap();
                    let size = size.unwrap();

                    let traits = vec![
                        trait_vector(name.clone(), size, component.name.clone())
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
                    let sources1: Vec<_> = args1.iter().map(|ident| Ident::from(format!("{}_src", ident))).collect();
                    let sources2 = sources1.clone();
                    let arg_list: Vec<_> = generics.iter()
                        .zip(args1.iter())
                        .map(|(gen, name)| quote! { #name: #gen })
                        .collect();
                    let constraints: Vec<_> = generics.iter()
                        .map(|gen| {
                            let comp = component.name.clone();
                            quote! { #gen: IntoValue<Output=#comp> }
                        })
                        .collect();

                    let graph_opt = args1.iter()
                        .fold(None, |curr, arg| match curr {
                            Some(tokens) => Some(quote! { #tokens.or(#arg.get_graph()) }),
                            None => Some(quote! { #arg.get_graph() }),
                        })
                        .unwrap();

                    quote! {
                        #[derive(Copy, Clone)]
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

                            fn get_graph(&self) -> Option<GraphRef> {
                                None
                            }

                            fn get_concrete(&self) -> Option<Self::Output> {
                                Some(*self)
                            }

                            fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
                                let mut graph = graph.borrow_mut();
                                graph.add_node(Node::Constant(TypedValue::#name( #( #fields ),* )))
                            }
                        }

                        #[inline]
                        pub fn #lower< #( #generics ),* >( #( #arg_list ),* ) -> Value<#name> where #( #constraints ),* {
                            if #( #args1.get_concrete().is_some() )&&* {
                                return Value::Concrete(#name( #( #args2.get_concrete().unwrap().0 ),* ));
                            }

                            let graph_opt = #graph_opt;
                            if let Some(graph_ref) = graph_opt {
                                #( let #sources1 = #args3.get_index(graph_ref.clone()); )*

                                let index = {
                                    let mut graph = graph_ref.borrow_mut();
                                    let index = graph.add_node(Node::Construct(TypeName::#upper));
                                    #( graph.add_edge(#sources2, index, #indices); )*
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
                },

                Category::MATRIX => {
                    let vector = component.unwrap();
                    let scalar = vector.clone().component.unwrap();
                    let size = size.unwrap();

                    let traits = vec![
                        trait_matrix(name.clone(), size, vector.name.clone(), scalar.name.clone())
                    ];

                    let ty = Ident::from(ty);
                    let mat_size = (size * size) as usize;
                    quote! {
                        #[derive(Copy, Clone)]
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

                        impl IntoValue for #name {
                            type Output = #name;

                            fn get_graph(&self) -> Option<GraphRef> {
                                None
                            }

                            fn get_concrete(&self) -> Option<Self::Output> {
                                Some(*self)
                            }

                            fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
                                let mut graph = graph.borrow_mut();
                                graph.add_node(Node::Constant(TypedValue::#name(self.0)))
                            }
                        }
                    }
                },

                _ => return None,
            };

            let upper = Ident::from(
                name.as_ref().to_string().to_uppercase()
            );

            Some(quote! {
                #decl

                impl Input<#name> for Shader {
                    #[inline]
                    fn input(&self, location: u32) -> Value<#name> {
                        let index = {
                            let mut graph = self.graph.borrow_mut();
                            graph.add_node(Node::Input(location, TypeName::#upper))
                        };

                        Value::Abstract {
                            graph: self.graph.clone(),
                            index,
                            ty: PhantomData,
                        }
                    }
                }

                impl Uniform<#name> for Shader {
                    #[inline]
                    fn uniform(&self, location: u32) -> Value<#name> {
                        let index = {
                            let mut graph = self.graph.borrow_mut();
                            graph.add_node(Node::Uniform(location, TypeName::#upper))
                        };

                        Value::Abstract {
                            graph: self.graph.clone(),
                            index,
                            ty: PhantomData,
                        }
                    }
                }

                impl Output<#name> for Shader {
                    #[inline]
                    fn output(&self, location: u32, source: Value<#name>) {
                        let source = source.get_index(self.graph.clone());

                        let mut graph = self.graph.borrow_mut();
                        let sink = graph.add_node(Node::Output(location, TypeName::#upper));
                        graph.add_edge(source, sink, 0);
                    }
                }
            })
        })
        .collect();

    /*for tokens in res.iter() {
        println!("{}", tokens);
    }*/

    res
}