//! GLSL Types declarations

use codegen::{
    defs::{all_types, Category, Type},
    operations::match_values,
};
use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};

fn trait_scalar(name: Ident, ty: Ident, (zero, one): (TokenTree, TokenTree)) -> TokenStream {
    quote! {
        impl Base for #name {
            fn zero() -> Self { #zero }
            fn one() -> Self { #one }
        }

        impl Scalar for #name {}

        impl ValueIter<#ty> for #name {
            type Iter = ::std::iter::Once<Value<#ty>>;
            fn iter(obj: &Self) -> Self::Iter {
                ::std::iter::once(obj.into())
            }
        }
    }
}

fn trait_numerical(name: Ident, pow_fn: Ident, pow_ty: Ident) -> TokenStream {
    quote! {
        impl Math for #name {}

        impl Numerical for #name {
            #[allow(clippy::cast_sign_loss)]
            fn pow(x: Self, y: Self) -> Self { x.#pow_fn(y as #pow_ty) }
        }
    }
}

fn trait_integer(name: Ident, is_signed: bool) -> TokenStream {
    quote! {
        impl Integer for #name {
            fn is_signed() -> bool { #is_signed }
        }
    }
}

fn trait_float(name: Ident, is_double: bool) -> TokenStream {
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
            fn ln(self) -> Self { self.ln() }
            fn abs(self) -> Self { self.abs() }
            fn two() -> Self { 2.0 }
            fn three() -> Self { 3.0 }
        }
    }
}

fn trait_vector(
    name: Ident,
    component_count: u32,
    component_type: Ident,
    (zero, one): (TokenTree, TokenTree),
) -> TokenStream {
    let fields_zero: Vec<_> = (0..component_count).map(|_| zero.clone()).collect();
    let fields_one: Vec<_> = (0..component_count).map(|_| one.clone()).collect();
    let obj_fields: Vec<_> = {
        (0..component_count)
            .map(|i| {
                let idx = Literal::u32_unsuffixed(i);
                quote! { obj.#idx.into() }
            })
            .collect()
    };

    let math = if component_type == "Bool" {
        quote!{}
    } else {
        quote! {
            impl Math for #name {}
        }
    };

    quote! {
        impl Base for #name {
            fn zero() -> Self { #name( #( #fields_zero ),* ) }
            fn one() -> Self { #name( #( #fields_one ),* ) }
        }

        #math

        impl Vector<#component_type> for #name {
            fn component_count() -> u32 { #component_count }
        }

        impl ValueIter<#component_type> for #name {
            type Iter = ::std::vec::IntoIter<Value<#component_type>>;
            fn iter(obj: &Self) -> Self::Iter {
                vec![ #( #obj_fields ),* ].into_iter()
            }
        }
    }
}

fn trait_matrix(
    name: Ident,
    column_count: u32,
    column_type: Ident,
    scalar_type: Ident,
    (zero, one): (TokenTree, TokenTree),
) -> TokenStream {
    let identity: Vec<TokenTree> = {
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
            fn identity() -> Self { #name( [ #( #identity, )* ] ) }
            fn column_count() -> u32 { #column_count }
        }

        impl ValueIter<#scalar_type> for #name {
            type Iter = ::std::vec::IntoIter<Value<#scalar_type>>;
            fn iter(obj: &Self) -> Self::Iter {
                let lst: Vec<_> = obj.0.into_iter().map(|s| s.into()).collect();
                lst.into_iter()
            }
        }
    }
}

fn type_values(ty: &str) -> (TokenTree, TokenTree) {
    match ty {
        "bool" => (
            TokenTree::Ident(Ident::new("false", Span::call_site())),
            TokenTree::Ident(Ident::new("true", Span::call_site())),
        ),

        "i32" | "u32" => (
            TokenTree::Literal(Literal::u8_unsuffixed(0)),
            TokenTree::Literal(Literal::u8_unsuffixed(1)),
        ),

        "f32" | "f64" => (
            TokenTree::Literal(Literal::f32_unsuffixed(0.0)),
            TokenTree::Literal(Literal::f32_unsuffixed(1.0)),
        ),

        _ => unreachable!(),
    }
}

fn type_scalar(name: &Ident, ty: &'static str) -> TokenStream {
    let mut traits = vec![trait_scalar(
        name.clone(),
        Ident::new(ty, Span::call_site()),
        type_values(ty),
    )];

    match ty {
        "bool" => {}

        "i32" | "u32" => {
            traits.push(trait_numerical(
                name.clone(),
                Ident::new("pow", Span::call_site()),
                Ident::new("u32", Span::call_site()),
            ));
            traits.push(trait_integer(name.clone(), ty == "i32"));
        }

        "f32" | "f64" => {
            traits.push(trait_numerical(
                name.clone(),
                Ident::new("powf", Span::call_site()),
                Ident::new(ty, Span::call_site()),
            ));
            traits.push(trait_float(name.clone(), ty == "f64"));
        }

        _ => unreachable!(),
    }

    let ty = Ident::new(ty, Span::call_site());
    quote! {
        pub type #name = #ty;
        #( #traits )*

        impl Into<Value<#name>> for #name {
            fn into(self) -> Value<Self> {
                Value::Concrete(self)
            }
        }
        impl<'a> Into<Value<#name>> for &'a #name {
            fn into(self) -> Value<#name> {
                Value::Concrete(*self)
            }
        }

        impl IntoValue for #name {
            type Output = Self;

            fn into_value(self) -> Value<Self> {
                Value::Concrete(self)
            }

            fn get_index(&self, mut graph: GraphRef) -> NodeIndex<u32> {
                graph.add_node(Node::Constant(TypedValue::#name(*self)))
            }
        }
    }
}

fn type_vector(
    name: &Ident,
    ty: &'static str,
    component: Option<Box<Type>>,
    size: Option<u32>,
) -> TokenStream {
    let component = component.unwrap();
    let size = size.unwrap();

    let traits = vec![trait_vector(
        name.clone(),
        size,
        component.name.clone(),
        type_values(ty),
    )];

    let ty = Ident::new(ty, Span::call_site());
    let types = (0..size).map(|_| ty.clone());

    let fields = (0..size).map(|i| {
        let id = Literal::u32_unsuffixed(i);
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
        let index = Literal::u32_unsuffixed(i);
        quote! { #i => &self.#index }
    });

    let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
    let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
    let indices: Vec<u32> = (0..size).collect();
    let generics: Vec<_> = (0..size)
        .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
        .collect();
    let args1: Vec<_> = generics
        .iter()
        .map(|gen| Ident::new(&gen.to_string().to_lowercase(), Span::call_site()))
        .collect();
    let args2 = args1.clone();
    let args3 = args1.clone();
    let arg_list: Vec<_> = {
        generics
            .iter()
            .zip(args1.iter())
            .map(|(gen, name)| quote! { #name: #gen })
            .collect()
    };
    let constraints: Vec<_> = {
        generics
            .iter()
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
            fn from(arr: Vec<#ty>) -> Self {
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
            fn into(self) -> Value<Self> {
                Value::Concrete(self)
            }
        }

        impl IntoValue for #name {
            type Output = Self;

            fn into_value(self) -> Value<Self> {
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

fn type_matrix(
    name: &Ident,
    ty: &'static str,
    component: Option<Box<Type>>,
    size: Option<u32>,
) -> TokenStream {
    let vector = component.unwrap();
    let scalar = vector.clone().component.unwrap();
    let size = size.unwrap();

    let traits = vec![trait_matrix(
        name.clone(),
        size,
        vector.name.clone(),
        scalar.name.clone(),
        type_values(ty),
    )];

    let ty = Ident::new(ty, Span::call_site());
    let mat_size = (size * size) as usize;

    let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
    let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
    let indices: Vec<u32> = (0..size).collect();
    let generics: Vec<_> = (0..size)
        .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
        .collect();
    let args1: Vec<_> = generics
        .iter()
        .map(|gen| Ident::new(&gen.to_string().to_lowercase(), Span::call_site()))
        .collect();
    let args2 = args1.clone();
    let args3: Vec<_> = {
        args1
            .iter()
            .flat_map(|ident| -> Vec<_> {
                (0..size)
                    .map(|index| {
                        let index = Literal::u32_unsuffixed(index);
                        quote! { #ident.#index }
                    })
                    .collect()
            })
            .collect()
    };
    let arg_list: Vec<_> = {
        generics
            .iter()
            .zip(args1.iter())
            .map(|(gen, name)| quote! { #name: #gen })
            .collect()
    };
    let constraints: Vec<_> = {
        generics
            .iter()
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
            fn into(self) -> Value<Self> {
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
            type Output = Self;

            fn into_value(self) -> Value<Self> {
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

pub fn type_structs() -> Vec<TokenStream> {
    all_types().iter()
        .filter_map(|ty| {
            let Type { name, category, ty, component, size, .. } = ty.clone();
            let decl = match category {
                Category::SCALAR => type_scalar(&name, ty),
                Category::VECTOR => type_vector(&name, ty, component, size),
                Category::MATRIX => type_matrix(&name, ty, component, size),
            };

            let upper = Ident::new(
                &name.to_string().to_string().to_uppercase(), Span::call_site()
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
