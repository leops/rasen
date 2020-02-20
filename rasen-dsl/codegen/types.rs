//! GLSL Types declarations

use codegen::defs::{all_types, Category, Type};
use proc_macro2::{Ident, Span, TokenStream};

fn type_scalar(name: &Ident, kind: &'static str) -> [TokenStream; 5] {
    let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
    let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
    let ty = Ident::new(kind, Span::call_site());

    let mut traits = Vec::new();

    match kind {
        "bool" => {}

        "i32" => {
            traits.push(quote! {
                impl GenType for #name {
                    #[inline]
                    fn zero() -> Self { 0 }
                    #[inline]
                    fn one() -> Self { 1 }
                    #[inline]
                    fn min(self, rhs: Self) -> Self { std::cmp::Ord::min(self, rhs) }
                    #[inline]
                    fn max(self, rhs: Self) -> Self { std::cmp::Ord::max(self, rhs) }
                }

                impl Numerical for #name {
                    #[inline]
                    fn pow(self, rhs: Self) -> Self { self.pow(std::convert::TryInto::try_into(rhs).unwrap()) }
                }
            });
        }
        "u32" => {
            traits.push(quote! {
                impl GenType for #name {
                    #[inline]
                    fn zero() -> Self { 0 }
                    #[inline]
                    fn one() -> Self { 1 }
                    #[inline]
                    fn min(self, rhs: Self) -> Self { std::cmp::Ord::min(self, rhs) }
                    #[inline]
                    fn max(self, rhs: Self) -> Self { std::cmp::Ord::max(self, rhs) }
                }

                impl Numerical for #name {
                    #[inline]
                    fn pow(self, rhs: Self) -> Self { self.pow(rhs) }
                }
            });
        }

        _ => {
            traits.push(quote! {
                impl GenType for #name {
                    #[inline]
                    fn zero() -> Self { 0.0 }
                    #[inline]
                    fn one() -> Self { 1.0 }
                    #[inline]
                    fn min(self, rhs: Self) -> Self { self.min(rhs) }
                    #[inline]
                    fn max(self, rhs: Self) -> Self { self.max(rhs) }
                }

                impl Numerical for #name {
                    #[inline]
                    fn pow(self, rhs: Self) -> Self { self.powf(rhs) }
                }

                impl Floating for #name {
                    #[inline]
                    fn sqrt(self) -> Self { self.sqrt() }
                    #[inline]
                    fn floor(self) -> Self { self.floor() }
                    #[inline]
                    fn ceil(self) -> Self { self.ceil() }
                    #[inline]
                    fn round(self) -> Self { self.round() }
                    #[inline]
                    fn sin(self) -> Self { self.sin() }
                    #[inline]
                    fn cos(self) -> Self { self.cos() }
                    #[inline]
                    fn tan(self) -> Self { self.tan() }
                    #[inline]
                    fn ln(self) -> Self { self.ln() }
                    #[inline]
                    fn abs(self) -> Self { self.abs() }
                }
            });
        }
    }

    let value = quote! {
        pub type #name = #ty;

        impl AsTypeName for #name {
            const TYPE_NAME: &'static TypeName = TypeName::#upper;
        }

        impl<C: Container<#name>> IntoValue<C> for #name {
            type Output = #name;

            #[inline]
            fn into_value(self) -> Value<C, #name> {
                C::#lower(self)
            }
        }

        #( #traits )*
    };

    let container = quote! {
        fn #lower(value: #name) -> Value<Self, #name>
        where
            Self: Container<#name>;
    };
    let context = quote! {
        Container<#name>
    };
    let parse = quote! {
        fn #lower(value: #name) -> Value<Self, #name> {
            with_graph(|graph| Value(graph.add_node(Node::Constant(TypedValue::#name(value)))))
        }
    };
    let execute = quote! {
        #[inline]
        fn #lower(value: #name) -> Value<Self, #name> {
            Value(value)
        }
    };

    [value, container, context, parse, execute]
}

fn type_vector(
    name: &Ident,
    ty: &'static str,
    component: Option<Box<Type>>,
    size: Option<u32>,
) -> [TokenStream; 5] {
    let component = component.unwrap();
    let comp = component.name.clone();
    let size = size.unwrap() as usize;

    let kind = ty;
    let ty = Ident::new(ty, Span::call_site());

    let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
    let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());

    let args1: Vec<_> = (0..size)
        .map(|i| Ident::new(&format!("arg_{}", i), Span::call_site()))
        .collect();
    let args2 = args1.clone();

    let parse_edges: Vec<_> = args1
        .iter()
        .enumerate()
        .map(|(index, ident)| {
            let ident = ident.clone();
            let index = index as u32;
            quote! { graph.add_edge(#ident.0, node, #index); }
        })
        .collect();

    let container_args: Vec<_> = {
        args1
            .iter()
            .map(|name| {
                let comp = comp.clone();
                quote! { #name: Value<Self, #comp> }
            })
            .collect()
    };
    let arg_list: Vec<_> = {
        args1
            .iter()
            .map(|name| {
                let comp = comp.clone();
                quote! { #name: impl IntoValue<C, Output=#comp> }
            })
            .collect()
    };
    {}

    let mut traits = Vec::new();

    if kind != "bool" {
        let values: Vec<_> = (0..size).map(|_| quote! { v }).collect();

        let self1: Vec<_> = (0..size)
            .map(|i| {
                let i = i as usize;
                quote! { self.0[#i] }
            })
            .collect();
        let rhs1: Vec<_> = (0..size)
            .map(|i| {
                let i = i as usize;
                quote! { rhs.0[#i] }
            })
            .collect();

        let self2 = self1.clone();
        let rhs2 = rhs1.clone();

        traits.push(quote! {
            impl GenType for #name {
                #[inline]
                fn zero() -> Self {
                    Self::spread(#ty::zero())
                }
                #[inline]
                fn one() -> Self {
                    Self::spread(#ty::one())
                }
                #[inline]
                fn min(self, rhs: Self) -> Self {
                    #name([ #( GenType::min(#self1, #rhs1) ),* ])
                }
                #[inline]
                fn max(self, rhs: Self) -> Self {
                    #name([ #( GenType::max(#self2, #rhs2) ),* ])
                }
            }

            impl Vector for #name {
                type Scalar = #ty;

                #[inline]
                fn spread(v: #ty) -> Self {
                    #name([ #( #values ),* ])
                }
            }
        });

        if kind != "i32" && kind != "u32" {
            let rhs_fields1: Vec<_> = (0..size)
                .map(|index| {
                    let index = index as usize;
                    quote! { rhs.0[#index] }
                })
                .collect();

            let fields1: Vec<_> = (0..size)
                .map(|index| {
                    let index = index as usize;
                    quote! { self.0[#index] }
                })
                .collect();

            let fields2 = fields1.clone();
            let fields3 = fields1.clone();

            if size == 3 {
                traits.push(quote! {
                    impl Vector3 for #name {
                        fn cross(&self, rhs: &Self) -> Self {
                            let #name(arg_0) = self;
                            let #name(arg_1) = rhs;

                            #name([
                                arg_0[1] * arg_1[2] - arg_1[1] * arg_0[2],
                                arg_0[2] * arg_1[0] - arg_1[2] * arg_0[0],
                                arg_0[0] * arg_1[1] - arg_1[0] * arg_0[1],
                            ])
                        }
                    }
                });
            }

            traits.push(quote! {
                impl VectorFloating for #name {
                    fn normalize(&self) -> Self {
                        let length = self.length();
                        #name([ #( #fields1 / length ),* ])
                    }

                    fn dot(&self, rhs: &Self) -> Self::Scalar {
                        #( #rhs_fields1 * #fields2 )+*
                    }

                    fn length_squared(&self) -> Self::Scalar {
                        #( #fields3.powi(2) )+*
                    }
                }
            });
        }
    }

    let value = quote! {
        #[derive(Copy, Clone, Debug)]
        pub struct #name( pub [ #ty ; #size ] );

        impl AsTypeName for #name {
            const TYPE_NAME: &'static TypeName = TypeName::#upper;
        }

        impl Index<u32> for #name {
            type Output = #ty;

            fn index(&self, index: u32) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        #( #traits )*

        #[inline]
        pub fn #lower<C: Context>( #( #arg_list ),* ) -> Value<C, #name> {
            <C as Container<#name>>::#lower( #( #args1.into_value() ),* )
        }
    };

    let parse_args = container_args.clone();
    let execute_args = container_args.clone();

    let container = quote! {
        fn #lower( #( #container_args ),* ) -> Value<Self, #name>
        where
            Self: Container<#comp> + Container<#name>;
    };
    let context = quote! {
        Container<#name>
    };

    let parse = quote! {
        fn #lower( #( #parse_args ),* ) -> Value<Self, #name> {
            with_graph(|graph| {
                let node = graph.add_node(Node::Construct(TypeName::#upper));
                #( #parse_edges )*
                Value(node)
            })
        }
    };
    let execute = quote! {
        #[inline]
        fn #lower( #( #execute_args ),* ) -> Value<Self, #name> {
            Value(#name([ #( #args2.0 ),* ]))
        }
    };

    [value, container, context, parse, execute]
}

fn type_matrix(
    name: &Ident,
    ty: &'static str,
    component: Option<Box<Type>>,
    size: Option<u32>,
) -> [TokenStream; 5] {
    let vector = component.unwrap();
    let comp = vector.name.clone();
    let size = size.unwrap();

    let ty = Ident::new(ty, Span::call_site());
    let mat_size = (size * size) as usize;

    let lower = Ident::new(&name.to_string().to_lowercase(), Span::call_site());
    let upper = Ident::new(&name.to_string().to_uppercase(), Span::call_site());

    let args: Vec<_> = (0..size)
        .map(|i| Ident::new(&format!("arg_{}", i), Span::call_site()))
        .collect();

    let parse_edges: Vec<_> = args
        .iter()
        .enumerate()
        .map(|(index, ident)| {
            let ident = ident.clone();
            let index = index as u32;
            quote! { graph.add_edge(#ident.0, node, #index); }
        })
        .collect();

    let execute_unwrap: Vec<_> = args
        .iter()
        .map(|ident| {
            let ident = quote! { (#ident.0).0 };
            let items: Vec<_> = (0..(size as usize))
                .map(|i| quote! { #ident[#i] })
                .collect();

            quote! { #( #items ),* }
        })
        .collect();

    let container_args: Vec<_> = {
        args.iter()
            .map(|name| {
                let comp = comp.clone();
                quote! { #name: Value<Self, #comp> }
            })
            .collect()
    };
    let arg_list: Vec<_> = {
        args.iter()
            .map(|name| {
                let comp = comp.clone();
                quote! { #name: impl IntoValue<C, Output=#comp> }
            })
            .collect()
    };
    {}

    let value = quote! {
        #[derive(Copy, Clone, Debug)]
        pub struct #name(pub [ #ty ; #mat_size ]);

        impl AsTypeName for #name {
            const TYPE_NAME: &'static TypeName = TypeName::#upper;
        }

        impl Index<u32> for #name {
            type Output = #ty;

            fn index(&self, index: u32) -> &Self::Output {
                &self.0[index as usize]
            }
        }

        impl Matrix for #name {
            fn inverse(self) -> Self {
                unimplemented!()
            }
        }

        #[inline]
        pub fn #lower<C: Context>( #( #arg_list ),* ) -> Value<C, #name> {
            <C as Container<#name>>::#lower( #( #args.into_value() ),* )
        }
    };

    let parse_args = container_args.clone();
    let execute_args = container_args.clone();

    let container = quote! {
        fn #lower( #( #container_args ),* ) -> Value<Self, #name>
        where
            Self: Container<#comp> + Container<#name>;
    };
    let context = quote! {
        Container<#name>
    };

    let parse = quote! {
        fn #lower( #( #parse_args ),* ) -> Value<Self, #name> {
            with_graph(|graph| {
                let node = graph.add_node(Node::Construct(TypeName::#upper));
                #( #parse_edges )*
                Value(node)
            })
        }
    };
    let execute = quote! {
        #[inline]
        fn #lower( #( #execute_args ),* ) -> Value<Self, #name> {
            Value(#name([ #( #execute_unwrap ),* ]))
        }
    };

    [value, container, context, parse, execute]
}

pub fn type_structs() -> Vec<[TokenStream; 5]> {
    all_types()
        .iter()
        .filter_map(|ty| {
            let Type {
                name,
                category,
                ty,
                component,
                size,
                ..
            } = ty.clone();
            let decl = match category {
                Category::SCALAR => type_scalar(&name, ty),
                Category::VECTOR => type_vector(&name, ty, component, size),
                Category::MATRIX => type_matrix(&name, ty, component, size),
            };

            Some(decl)
        })
        .collect()
}
