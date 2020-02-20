//! GLSL Operation declarations

use proc_macro2::{Ident, Span, TokenStream};
use std::collections::{hash_map::Entry, HashMap};

enum Generic {
    Container,
    Other(char),
}

impl Generic {
    fn generic(&self) -> Option<char> {
        match self {
            Generic::Container => None,
            Generic::Other(gen) => Some(*gen),
        }
    }

    fn tokens(&self) -> TokenStream {
        match self {
            Generic::Container => {
                quote! { T }
            }
            Generic::Other(gen) => {
                let name = Ident::new(&gen.to_string(), Span::call_site());
                quote! { #name }
            }
        }
    }
}

enum ArgType {
    Generic(Generic),
    Value(Box<ArgType>),
    Associated {
        generic: Generic,
        trait_: TokenStream,
        name: &'static str,
    },
}

impl ArgType {
    fn generic(&self) -> Option<char> {
        match self {
            ArgType::Generic(gen) => gen.generic(),
            ArgType::Value(inner) => inner.generic(),
            ArgType::Associated { generic, .. } => generic.generic(),
        }
    }

    fn tokens(&self, is_trait: bool, is_ret: bool) -> TokenStream {
        match self {
            ArgType::Generic(gen) => gen.tokens(),
            ArgType::Value(inner) => {
                let inner = inner.tokens(is_trait, is_ret);
                let ctx = if is_trait {
                    quote! { Self }
                } else {
                    quote! { C }
                };

                if is_trait || is_ret {
                    quote! { Value<#ctx, #inner> }
                } else {
                    quote! { impl IntoValue<#ctx, Output = #inner> }
                }
            }
            ArgType::Associated {
                generic,
                trait_,
                name,
            } => {
                let generic = generic.tokens();
                let name = Ident::new(name, Span::call_site());
                quote! { <#generic as #trait_>::#name }
            }
        }
    }

    fn constraints(&self, context: &Context) -> Constraints {
        match self {
            ArgType::Value(inner) => {
                let tokens = inner.tokens(context.is_trait(), true);

                let mut inner = inner.constraints(context);
                inner.add(quote! { #tokens }, quote! { Copy });

                match context {
                    Context::Trait => inner.add(quote! { Self }, quote! { Container<#tokens> }),
                    Context::Func => inner.add(quote! { C }, quote! { Container<#tokens> }),
                    _ => {}
                }

                inner
            }

            ArgType::Associated {
                generic, trait_, ..
            } => {
                let generic = generic.tokens();
                Constraints::of(quote! { #generic }, trait_.clone())
            }

            _ => Constraints::default(),
        }
    }
}

struct Argument {
    name: &'static str,
    ty: ArgType,
}

#[derive(Hash, Eq, PartialEq)]
enum Context {
    Trait,
    Parse,
    Execute,
    Func,
}

impl Context {
    fn is_trait(&self) -> bool {
        if let Context::Func = self {
            false
        } else {
            true
        }
    }
}

struct Constraint {
    key: TokenStream,
    values: Vec<TokenStream>,
}

#[derive(Default)]
struct Constraints {
    inner: HashMap<String, Constraint>,
}

impl Constraints {
    fn of(key: TokenStream, value: TokenStream) -> Self {
        let mut inner = HashMap::new();
        inner.insert(
            key.to_string(),
            Constraint {
                key,
                values: vec![value],
            },
        );
        Constraints { inner }
    }

    fn add(&mut self, key: TokenStream, value: TokenStream) {
        self.inner
            .entry(key.to_string())
            .or_insert_with(|| Constraint {
                key: key.clone(),
                values: Vec::new(),
            })
            .values
            .push(value);
    }

    fn extend(&mut self, other: Constraints) {
        for (key, value) in other.inner {
            match self.inner.entry(key) {
                Entry::Occupied(mut entry) => {
                    let entry = entry.get_mut();
                    entry.values.extend(value.values);
                }
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
    }
}

fn operation(
    name: &str,
    args: &[Argument],
    result: ArgType,
    constraint_list: &[Constraint],
    node: TokenStream,
    edges: &[&str],
    implementation: TokenStream,
) -> [TokenStream; 4] {
    let fn_name = Ident::new(name, Span::call_site());

    let mut output = [
        TokenStream::new(),
        TokenStream::new(),
        TokenStream::new(),
        TokenStream::new(),
    ];

    let generics: Vec<_> = args
        .into_iter()
        .filter_map(|arg| {
            arg.ty
                .generic()
                .map(|name| Ident::new(&name.to_string(), Span::call_site()))
        })
        .collect();

    for (index, context) in [
        Context::Trait,
        Context::Parse,
        Context::Execute,
        Context::Func,
    ]
    .iter()
    .enumerate()
    {
        let generics = generics.clone();

        let mut constraints = Constraints::default();

        for constraint in constraint_list {
            for value in &constraint.values {
                constraints.add(constraint.key.clone(), value.clone());
            }
        }

        for arg in args {
            constraints.extend(arg.ty.constraints(context));
        }

        constraints.extend(result.constraints(context));

        let constraints: Vec<_> = constraints
            .inner
            .values()
            .map(|constraint| {
                let key = constraint.key.clone();
                let values = constraint.values.clone();
                quote! { #key: #( #values )+* }
            })
            .collect();

        let is_trait = context.is_trait();

        let fn_args: Vec<_> = args
            .into_iter()
            .map(|arg| {
                let name = Ident::new(&arg.name, Span::call_site());
                let ty = arg.ty.tokens(is_trait, false);
                quote! {
                    #name: #ty
                }
            })
            .collect();

        let result = result.tokens(is_trait, true);

        output[index] = match context {
            Context::Trait => quote! {
                fn #fn_name< #( #generics ),* >( #( #fn_args ),* ) -> #result where #( #constraints ),*;
            },

            Context::Parse => {
                let edges: Vec<_> = edges
                    .into_iter()
                    .enumerate()
                    .map(|(index, name)| {
                        let ident = Ident::new(name, Span::call_site());
                        let index = index as u32;

                        quote! { graph.add_edge(#ident.0, node, #index); }
                    })
                    .collect();

                quote! {
                    fn #fn_name< #( #generics ),* >( #( #fn_args ),* ) -> #result where #( #constraints, )* {
                        with_graph(|graph| {
                            let node = graph.add_node(#node);
                            #( #edges )*
                            Value(node)
                        })
                    }
                }
            }

            Context::Execute => {
                let args_unwrap: Vec<_> = args
                    .into_iter()
                    .filter_map(|arg| {
                        if let ArgType::Value(_) = arg.ty {
                            let name = Ident::new(&arg.name, Span::call_site());
                            Some(quote! { let Value(#name) = #name; })
                        } else {
                            None
                        }
                    })
                    .collect();

                quote! {
                    #[inline]
                    fn #fn_name< #( #generics ),* >( #( #fn_args ),* ) -> #result where #( #constraints, )* {
                        #( #args_unwrap )*
                        Value({ #implementation })
                    }
                }
            }

            Context::Func => {
                let arg_names: Vec<_> = args
                    .into_iter()
                    .map(|arg| {
                        let name = Ident::new(&arg.name, Span::call_site());
                        if let ArgType::Value(_) = arg.ty {
                            quote! { #name.into_value() }
                        } else {
                            quote! { #name }
                        }
                    })
                    .collect();

                quote! {
                    #[inline]
                    pub fn #fn_name<C, T, #( #generics ),*>( #( #fn_args ),* ) -> #result where #( #constraints, )* {
                        <C as Container<T>>::#fn_name( #( #arg_names ),* )
                    }
                }
            }
        };
    }

    output
}

pub fn impl_operations() -> Vec<[TokenStream; 4]> {
    vec![
        operation(
            "index",
            &[
                Argument {
                    name: "container",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "index",
                    ty: ArgType::Generic(Generic::Other('I')),
                },
            ],
            ArgType::Value(Box::new(ArgType::Associated {
                generic: Generic::Container,
                trait_: quote! { Index<I> },
                name: "Output",
            })),
            &[
                Constraint {
                    key: quote! { u32 },
                    values: vec![quote! { From<I> }],
                },
                Constraint {
                    key: quote! { T::Output },
                    values: vec![quote! { Sized }],
                },
            ],
            quote! {
                Node::Extract(index.into())
            },
            &["container"],
            quote! {
                container[index]
            },
        ),
        operation(
            "normalize",
            &[Argument {
                name: "vector",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[
                Constraint {
                    key: quote! { T },
                    values: vec![quote! { VectorFloating }],
                },
                Constraint {
                    key: quote! { <T as Vector>::Scalar },
                    values: vec![quote! { Floating }],
                },
            ],
            quote! {
                Node::Normalize
            },
            &["vector"],
            quote! {
                vector.normalize()
            },
        ),
        operation(
            "dot",
            &[
                Argument {
                    name: "a",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "b",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Associated {
                generic: Generic::Container,
                trait_: quote! { Vector },
                name: "Scalar",
            })),
            &[
                Constraint {
                    key: quote! { T },
                    values: vec![quote! { VectorFloating }],
                },
                Constraint {
                    key: quote! { <T as Vector>::Scalar },
                    values: vec![quote! { Floating }],
                },
            ],
            quote! {
                Node::Dot
            },
            &["a", "b"],
            quote! {
                a.dot(&b)
            },
        ),
        operation(
            "clamp",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "min",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "max",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Numerical }],
            }],
            quote! {
                Node::Clamp
            },
            &["x", "min", "max"],
            quote! {
                x.max(min).min(max)
            },
        ),
        operation(
            "cross",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "y",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Vector3 }],
            }],
            quote! {
                Node::Cross
            },
            &["x", "y"],
            quote! {
                x.cross(&y)
            },
        ),
        operation(
            "floor",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Floor
            },
            &["val"],
            quote! {
                val.floor()
            },
        ),
        operation(
            "ceil",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Ceil
            },
            &["val"],
            quote! {
                val.ceil()
            },
        ),
        operation(
            "round",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Round
            },
            &["val"],
            quote! {
                val.round()
            },
        ),
        operation(
            "sin",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Sin
            },
            &["val"],
            quote! {
                val.sin()
            },
        ),
        operation(
            "cos",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Cos
            },
            &["val"],
            quote! {
                val.cos()
            },
        ),
        operation(
            "tan",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Tan
            },
            &["val"],
            quote! {
                val.tan()
            },
        ),
        operation(
            "pow",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "y",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Numerical }],
            }],
            quote! {
                Node::Pow
            },
            &["x", "y"],
            quote! {
                x.pow(y)
            },
        ),
        operation(
            "min",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "y",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Numerical }],
            }],
            quote! {
                Node::Min
            },
            &["x", "y"],
            quote! {
                x.min(y)
            },
        ),
        operation(
            "max",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "y",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Numerical }],
            }],
            quote! {
                Node::Max
            },
            &["x", "y"],
            quote! {
                x.max(y)
            },
        ),
        operation(
            "length",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Associated {
                generic: Generic::Container,
                trait_: quote! { Vector },
                name: "Scalar",
            })),
            &[
                Constraint {
                    key: quote! { T },
                    values: vec![quote! { VectorFloating }],
                },
                Constraint {
                    key: quote! { <T as Vector>::Scalar },
                    values: vec![quote! { Floating }],
                },
            ],
            quote! {
                Node::Length
            },
            &["val"],
            quote! {
                val.length()
            },
        ),
        operation(
            "distance",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "y",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Associated {
                generic: Generic::Container,
                trait_: quote! { Vector },
                name: "Scalar",
            })),
            &[
                Constraint {
                    key: quote! { T },
                    values: vec![quote! { VectorFloating }, quote! { Sub<T, Output = T> }],
                },
                Constraint {
                    key: quote! { <T as Vector>::Scalar },
                    values: vec![quote! { Floating }],
                },
            ],
            quote! {
                Node::Distance
            },
            &["x", "y"],
            quote! {
                (x - y).length()
            },
        ),
        operation(
            "reflect",
            &[
                Argument {
                    name: "i",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "n",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[
                Constraint {
                    key: quote! { T },
                    values: vec![quote! { VectorFloating }, quote! { Sub<T, Output = T> }],
                },
                Constraint {
                    key: quote! { <T as Vector>::Scalar },
                    values: vec![
                        quote! { Floating },
                        quote! { Add<T::Scalar, Output = T::Scalar> },
                        quote! { Mul<T, Output = T> },
                    ],
                },
            ],
            quote! {
                Node::Reflect
            },
            &["i", "n"],
            quote! {
                let dot = n.dot(&i);
                i - (dot + dot) * n
            },
        ),
        operation(
            "refract",
            &[
                Argument {
                    name: "i",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "n",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "eta",
                    ty: ArgType::Value(Box::new(ArgType::Associated {
                        generic: Generic::Container,
                        trait_: quote! { Vector },
                        name: "Scalar",
                    })),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[
                Constraint {
                    key: quote! { T },
                    values: vec![quote! { VectorFloating }, quote! { Sub<T, Output=T> }],
                },
                Constraint {
                    key: quote! { <T as Vector>::Scalar },
                    values: vec![
                        quote! { Floating },
                        quote! { Add<T::Scalar, Output=T::Scalar> },
                        quote! { Sub<T::Scalar, Output=T::Scalar> },
                        quote! { Mul<T::Scalar, Output=T::Scalar> },
                        quote! { Mul<T, Output=T> },
                        quote! { PartialOrd<T::Scalar> },
                    ],
                },
            ],
            quote! {
                Node::Refract
            },
            &["i", "n", "eta"],
            quote! {
                let one = <T::Scalar as GenType>::one();
                let k = one - eta * eta * (one - n.dot(&i) * n.dot(&i));
                if k < <T::Scalar as GenType>::zero() {
                    T::zero()
                } else {
                    eta * i - (eta * n.dot(&i) + k.sqrt()) * n
                }
            },
        ),
        operation(
            "mix",
            &[
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "y",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "a",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![
                    quote! { GenType },
                    quote! { Add<T, Output = T> },
                    quote! { Sub<T, Output = T> },
                    quote! { Mul<T, Output = T> },
                ],
            }],
            quote! {
                Node::Mix
            },
            &["x", "y", "a"],
            quote! {
                x * (T::one() - a) + y * a
            },
        ),
        operation(
            "sqrt",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Sqrt
            },
            &["val"],
            quote! {
                val.sqrt()
            },
        ),
        operation(
            "log",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Log
            },
            &["val"],
            quote! {
                val.ln()
            },
        ),
        operation(
            "abs",
            &[Argument {
                name: "val",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Floating }],
            }],
            quote! {
                Node::Abs
            },
            &["val"],
            quote! {
                val.abs()
            },
        ),
        operation(
            "smoothstep",
            &[
                Argument {
                    name: "edge0",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "edge1",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![
                    quote! { GenType },
                    quote! { Add<T, Output=T> },
                    quote! { Sub<T, Output=T> },
                    quote! { Mul<T, Output=T> },
                    quote! { Div<T, Output=T> },
                ],
            }],
            quote! {
                Node::Smoothstep
            },
            &["edge0", "edge1", "x"],
            quote! {
                let two = T::one() + T::one();
                let three = two + T::one();

                let t = ((x - edge0) / (edge1 - edge0)).max(T::zero()).min(T::one());
                t * t * (three - two * t)
            },
        ),
        operation(
            "inverse",
            &[Argument {
                name: "v",
                ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            }],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { Matrix }],
            }],
            quote! {
                Node::Inverse
            },
            &["v"],
            quote! {
                v.inverse()
            },
        ),
        operation(
            "step",
            &[
                Argument {
                    name: "edge",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
                Argument {
                    name: "x",
                    ty: ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
                },
            ],
            ArgType::Value(Box::new(ArgType::Generic(Generic::Container))),
            &[Constraint {
                key: quote! { T },
                values: vec![quote! { GenType }, quote! { PartialOrd<T> }],
            }],
            quote! {
                Node::Step
            },
            &["edge", "x"],
            quote! {
                if x < edge {
                    T::zero()
                } else {
                    T::one()
                }
            },
        ),
    ]
}
