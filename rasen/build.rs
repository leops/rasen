#![recursion_limit = "128"]
#![warn(clippy::all, clippy::pedantic)]

#[macro_use]
extern crate quote;
extern crate proc_macro2;

use std::{collections::HashMap, env, fs::File, io::Write, path::Path, str::FromStr};

use proc_macro2::{Ident, Span, TokenStream};

static INTS: [(&'static str, &'static str, &'static str); 3] = [
    ("Bool", "b", "bool"),
    ("Int", "i", "i32"),
    ("UInt", "u", "u32"),
];
static FLOATS: [(&'static str, &'static str, &'static str); 2] =
    [("Float", "", "f32"), ("Double", "d", "f64")];

static SAMPLERS: [(&'static str, &'static str); 3] = [("", "FLOAT"), ("I", "INT"), ("U", "UINT")];
static DIMENSIONS: [&'static str; 6] = ["1D", "2D", "3D", "Cube", "Rect", "Buffer"];

enum ConstType {
    Vec(u32, Ident),
    Mat(u32, Ident),
    Sampler(Ident, Ident),
}

fn types(out_dir: &str) {
    let builder = TokenStream::from_str("$builder").unwrap();
    let constant = TokenStream::from_str("$constant").unwrap();

    let mut const_types = HashMap::new();
    let mut from_string_arms = Vec::new();
    let mut typed_variants = Vec::new();
    let mut type_name_arms = Vec::new();
    let mut typed_value_from = Vec::new();
    let mut register_constant_arms = Vec::new();

    for &(name, _, ty) in INTS.iter().chain(FLOATS.iter()) {
        let const_name = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
        let glsl_name = name.to_string().to_lowercase();
        let name = Ident::new(&name, Span::call_site());

        from_string_arms.push(quote! {
            #glsl_name => Self::#const_name,
        });

        type_name_arms.push(quote! {
            TypedValue::#name(..) => TypeName::#const_name
        });

        let ty_1 = Ident::new(ty, Span::call_site());
        let ty_2 = Ident::new(ty, Span::call_site());
        typed_value_from.push(quote! {
            impl From<#ty_1> for TypedValue {
                fn from(value: #ty_2) -> Self {
                    TypedValue::#name(value)
                }
            }
        });

        let register_val = match ty {
            "bool" => quote! {
                if val {
                    #builder.module.types_global_values.push(
                        Instruction::new(
                            Op::ConstantTrue,
                            Some(res_type),
                            Some(res_id),
                            Vec::new()
                        )
                    );
                } else {
                    #builder.module.types_global_values.push(
                        Instruction::new(
                            Op::ConstantFalse,
                            Some(res_type),
                            Some(res_id),
                            Vec::new()
                        )
                    );
                }
            },

            "i32" | "u32" => quote! {
                #builder.module.types_global_values.push(
                    Instruction::new(
                        Op::Constant,
                        Some(res_type),
                        Some(res_id),
                        vec![
                            #[allow(clippy::cast_sign_loss)]
                            Operand::LiteralInt32(val as u32),
                        ]
                    )
                );
            },

            "f32" => quote! {
                #builder.module.types_global_values.push(
                    Instruction::new(
                        Op::Constant,
                        Some(res_type),
                        Some(res_id),
                        vec![
                            Operand::LiteralFloat32(val),
                        ]
                    )
                );
            },
            "f64" => quote! {
                #builder.module.types_global_values.push(
                    Instruction::new(
                        Op::Constant,
                        Some(res_type),
                        Some(res_id),
                        vec![
                            Operand::LiteralFloat64(val),
                        ]
                    )
                );
            },

            _ => unreachable!(),
        };

        register_constant_arms.push(quote! {
            TypedValue::#name(val) => {
                let res_type = #builder.register_type(#constant.to_type_name());
                let res_id = #builder.get_id();
                #register_val
                Ok(res_id)
            },
        });

        let ty = Ident::new(&ty, Span::call_site());
        typed_variants.push(quote! {
            #name(#ty)
        });
    }

    for size in 2u32..5u32 {
        for &(name, prefix, ty) in INTS.iter().chain(FLOATS.iter()) {
            let type_variant = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            let const_name = Ident::new(&type_variant.to_uppercase(), Span::call_site());
            let glsl_name = type_variant.to_lowercase();
            let type_variant = Ident::new(&type_variant, Span::call_site());

            let const_ty = Ident::new(&name.to_string().to_uppercase(), Span::call_site());
            let name = Ident::new(&name, Span::call_site());

            const_types.insert(const_name.clone(), ConstType::Vec(size, const_ty));

            from_string_arms.push(quote! {
                #glsl_name => Self::#const_name,
            });

            let tuple_fields_1: Vec<_> = (0..size)
                .map(|_| Ident::new(&ty, Span::call_site()))
                .collect();
            let tuple_fields_2 = tuple_fields_1.clone();
            let tuple_fields_3 = tuple_fields_1.clone();
            typed_variants.push(quote! {
                #type_variant( #( #tuple_fields_1 ),* )
            });

            let tuple_values_1: Vec<_> = (0..size)
                .map(|i| Ident::new(&format!("v{}", i), Span::call_site()))
                .collect();
            let tuple_values_2 = tuple_values_1.clone();
            typed_value_from.push(quote! {
                impl From<( #( #tuple_fields_2 ),* )> for TypedValue {
                    fn from(( #( #tuple_values_1 ),* ): ( #( #tuple_fields_3 ),* )) -> Self {
                        TypedValue::#type_variant( #( #tuple_values_2 ),* )
                    }
                }
            });

            type_name_arms.push(quote! {
                TypedValue::#type_variant(..) => TypeName::#const_name
            });

            let fields: Vec<_> = (0..size)
                .map(|i| Ident::new(&format!("f{}", i), Span::call_site()))
                .collect();
            let field_ids: Vec<_> = (0..size)
                .map(|i| Ident::new(&format!("f{}_id", i), Span::call_site()))
                .collect();

            let register_fields: Vec<_> = field_ids
                .iter()
                .zip(fields.iter())
                .map(|(id, field)| {
                    quote! {
                        let #id = #builder.register_constant(&TypedValue::#name(#field))?;
                    }
                })
                .collect();

            register_constant_arms.push(quote! {
                TypedValue::#type_variant(#( #fields ),*) => {
                    #( #register_fields )*

                    let res_type = #builder.register_type(#constant.to_type_name());
                    let res_id = #builder.get_id();

                    #builder.module.types_global_values.push(
                        Instruction::new(
                            Op::ConstantComposite,
                            Some(res_type),
                            Some(res_id),
                            vec![
                                #( Operand::IdRef( #field_ids ) ),*
                            ]
                        )
                    );

                    Ok(res_id)
                },
            });
        }

        for &(_, prefix, ty) in &FLOATS {
            let ty = Ident::new(&ty, Span::call_site());
            let const_ty = Ident::new(
                &format!("{}VEC{}", prefix.to_string().to_uppercase(), size),
                Span::call_site(),
            );

            let type_variant = format!("{}Mat{}", prefix.to_string().to_uppercase(), size);
            let const_ident = Ident::new(&type_variant.to_uppercase(), Span::call_site());
            let glsl_type = type_variant.to_lowercase();
            let type_variant = Ident::new(&type_variant, Span::call_site());

            const_types.insert(const_ident.clone(), ConstType::Mat(size, const_ty));

            from_string_arms.push(quote! {
                #glsl_type => Self::#const_ident,
            });

            let arr_size = (size * size) as usize;
            typed_variants.push(quote! {
                #type_variant([#ty; #arr_size])
            });

            typed_value_from.push(quote! {
                impl From<[#ty; #arr_size]> for TypedValue {
                    fn from(value: [#ty; #arr_size]) -> Self {
                        TypedValue::#type_variant(value)
                    }
                }
            });

            type_name_arms.push(quote! {
                TypedValue::#type_variant(..) => TypeName::#const_ident
            });
        }
    }

    for dim in &DIMENSIONS {
        let dim_upper = if *dim == "Rect" {
            String::from("2DRECT")
        } else {
            dim.to_string().to_uppercase()
        };

        let dim = Ident::new(&format!("Dim{}", dim), Span::call_site());
        for &(prefix, ty) in &SAMPLERS {
            let ty = Ident::new(&ty, Span::call_site());
            let name = Ident::new(
                &format!("{}SAMPLER{}", prefix, dim_upper),
                Span::call_site(),
            );

            const_types.insert(name, ConstType::Sampler(ty, dim.clone()));
        }
    }

    let ptr_arms: Vec<_> = {
        const_types
            .keys()
            .filter_map(|name| {
                use proc_macro2::{Ident, TokenStream};

                fn as_pattern(
                    const_types: &HashMap<Ident, ConstType>,
                    name: &Ident,
                ) -> Result<TokenStream, ()> {
                    let ty = if let Some(ty) = const_types.get(name) {
                        ty
                    } else {
                        let name = name.to_string();
                        return Ok(match &name as &str {
                            "BOOL" => quote! { TypeName::Bool },
                            "INT" => quote! { TypeName::Int(true) },
                            "UINT" => quote! { TypeName::Int(false) },
                            "FLOAT" => quote! { TypeName::Float(false) },
                            "DOUBLE" => quote! { TypeName::Float(true) },

                            other => panic!("{:?}", other),
                        });
                    };

                    Ok(match *ty {
                        ConstType::Vec(size, ref const_ty) => {
                            let const_ty = as_pattern(const_types, const_ty)?;
                            quote! {
                                TypeName::Vec(#size, & #const_ty)
                            }
                        }
                        ConstType::Mat(size, ref const_ty) => {
                            let const_ty = as_pattern(const_types, const_ty)?;
                            quote! {
                                TypeName::Mat(#size, & #const_ty)
                            }
                        }
                        ConstType::Sampler(ref ty, ref dim) => {
                            let ty = as_pattern(const_types, ty)?;
                            let dim = dim.clone();
                            quote! {
                                TypeName::Sampler(& #ty, Dim::#dim)
                            }
                        }
                    })
                }

                as_pattern(&const_types, name).ok().map(|pattern| {
                    let ptr_name = Ident::new(&format!("{}_PTR", name), Span::call_site());
                    quote! {
                        #pattern => Self::#ptr_name,
                    }
                })
            })
            .collect()
    };

    let const_types: Vec<_> = {
        const_types
            .into_iter()
            .map(|(name, ty)| {
                let value = match ty {
                    ConstType::Vec(size, const_ty) => quote! {
                        TypeName::Vec(#size, Self::#const_ty)
                    },
                    ConstType::Mat(size, const_ty) => quote! {
                        TypeName::Mat(#size, Self::#const_ty)
                    },
                    ConstType::Sampler(ty, dim) => quote! {
                        TypeName::Sampler(Self::#ty, Dim::#dim)
                    },
                };

                let ptr_name = Ident::new(&format!("{}_PTR", name), Span::call_site());
                quote! {
                    pub const #name: &'static Self = & #value;
                    const #ptr_name: &'static Self = &TypeName::_Pointer(Self::#name);
                }
            })
            .collect()
    };

    let type_name_impl = quote! {
        #[allow(clippy::unseparated_literal_suffix)]
        impl TypeName {
            #( #const_types )*

            #[inline]
            pub fn from_string(ty: &str) -> Option<&'static Self> {
                Some(match ty {
                    #( #from_string_arms )*
                    _ => return None
                })
            }

            #[inline]
            pub(crate) fn as_ptr(&'static self) -> &'static Self {
                match *self {
                    TypeName::Float(false) => Self::FLOAT_PTR,
                    #( #ptr_arms )*
                    ref other => panic!("Missing as_ptr implementation: {:?}", other),
                }
            }
        }
    };

    let typed_value = quote! {
        /// Holder for a GLSL value and a type
        #[derive(Debug)]
        #[allow(clippy::unseparated_literal_suffix)]
        pub enum TypedValue {
            #( #typed_variants ),*
        }
    };

    let typed_value_impl = quote! {
        impl TypedValue {
            #[inline]
            pub fn to_type_name(&self) -> &'static TypeName {
                match *self {
                    #( #type_name_arms ),*
                }
            }
        }

        #( #typed_value_from )*
    };

    let path_types = Path::new(out_dir).join("types.rs");
    let mut f_types = File::create(&path_types).unwrap();

    write!(
        f_types,
        "{}\n{}\n{}",
        type_name_impl, typed_value, typed_value_impl
    )
    .unwrap();

    let register_constant = quote! {
        match *#constant {
            #( #register_constant_arms )*
            _ => Err(ErrorKind::UnsupportedConstant(#constant.to_type_name()))
        }
    };

    let path_builder = Path::new(out_dir).join("builder.rs");
    let mut f_builder = File::create(&path_builder).unwrap();

    write!(f_builder,
        "macro_rules! impl_register_constant {{\n( $builder:expr, $constant:expr ) => {{\n{}\n}};\n}}",
        register_constant
    ).unwrap();
}

const NODES: &'static [(&'static str, &'static str, &'static str)] = &[
    (
        "Normalize",
        "Normalize a vector",
        "Takes a single parameter",
    ),
    (
        "Add",
        "Add some values",
        "This node takes at least 2 parameters (left-associative)",
    ),
    (
        "Subtract",
        "Subtract a value from another",
        "This node takes at least 2 parameters (left-associative)",
    ),
    (
        "Multiply",
        "Multiply some values",
        "This node takes at least 2 parameters (left-associative)",
    ),
    (
        "Divide",
        "Divide a value by another",
        "This node takes at least 2 parameters (left-associative)",
    ),
    (
        "Modulus",
        "Compute the modulus of a value by another",
        "Takes 2 parameters",
    ),
    (
        "Clamp",
        "Clamp a value in a range",
        "Takes 3 parameters: the value to be clamped, the minimum, and the maximum",
    ),
    (
        "Dot",
        "Compute the dot product of 2 vectors",
        "Takes 2 parameters",
    ),
    (
        "Cross",
        "Compute the cross product of 2 vectors",
        "Takes 2 parameters",
    ),
    (
        "Floor",
        "Round a number to the largest lower or equal integer",
        "Takes a single parameter",
    ),
    (
        "Ceil",
        "Round a number to the nearest integer",
        "Takes a single parameter",
    ),
    (
        "Round",
        "Round a number to the smallest higher or equal integer",
        "Takes a single parameter",
    ),
    (
        "Sin",
        "Compute the sinus of an angle in radians",
        "Takes a single parameter",
    ),
    (
        "Cos",
        "Compute the cosinus of an angle in radians",
        "Takes a single parameter",
    ),
    (
        "Tan",
        "Compute the tangent of an angle in radians",
        "Takes a single parameter",
    ),
    ("Pow", "Raise a number to a power", "Takes 2 parameters"),
    (
        "Min",
        "Returns the smallest value of all its arguments",
        "This node takes at least 2 parameters",
    ),
    (
        "Max",
        "Return the greatest value of all its arguments",
        "This node takes at least 2 parameters",
    ),
    (
        "Length",
        "Computes the length of a vector",
        "Takes a single parameter",
    ),
    (
        "Distance",
        "Computes the distance between 2 points",
        "Takes 2 parameters",
    ),
    (
        "Reflect",
        "Reflect a vector against a surface normal",
        "Takes 2 parameters",
    ),
    (
        "Refract",
        "Computes the refraction of a vector using a surface normal and a refraction indice",
        "Takes 3 parameters",
    ),
    (
        "Mix",
        "Computes a linear interpolation between two values",
        "Takes 3 parameters",
    ),
    (
        "Sample",
        "Samples a texture using a coordinates vector",
        "Takes 2 or 3 parameters: the texture sampler, the coordinates, and an optional LOD bias",
    ),
    (
        "Sqrt",
        "Compute the square root of a value",
        "Takes one parameter",
    ),
    (
        "Log",
        "Compute the natural logarithm of a value",
        "Takes one parameter",
    ),
    (
        "Abs",
        "Compute the absolute value of a float",
        "Takes one parameter",
    ),
    (
        "Smoothstep",
        "Perform Hermite interpolation between two values",
        "Takes three parameter",
    ),
    (
        "Inverse",
        "Calculate the inverse of a matrix",
        "Takes one parameter",
    ),
    (
        "Step",
        "Generate a step function by comparing two values",
        "Takes two parameter",
    ),
];

fn nodes(out_dir: &str) {
    let mut node_variants = Vec::new();
    let mut to_string_arms = Vec::new();
    let mut from_string_arms = Vec::new();

    for &(name, desc, params) in NODES {
        let ident = Ident::new(&name.to_string(), Span::call_site());
        node_variants.push(quote! {
            #[doc = #desc]
            #[doc = ""]
            #[doc = #params]
            #ident
        });

        to_string_arms.push(quote! {
            Node::#ident => #name
        });

        from_string_arms.push(quote! {
            #name => Node::#ident
        });
    }

    let node = quote! {
        /// All the supported operations
        #[derive(Debug)]
        pub enum Node {
            /// Create an input with a location and a type
            ///
            /// Incoming values from other nodes are ignored
            Input(u32, &'static TypeName, VariableName),

            /// Create a uniform with a location and a type
            ///
            /// Incoming values from other nodes are ignored
            Uniform(u32, &'static TypeName, VariableName),

            /// Create an output with a location and a type
            ///
            /// Doesn't need to be an output of the graph, but all the outputs should use this type
            Output(u32, &'static TypeName, VariableName),

            /// Declare a new constant
            ///
            /// Incoming values from other nodes are ignored
            Constant(TypedValue),

            /// Build a composite object (only vectors are supported at the moment)
            ///
            /// Uses 2, 3 or 4 arguments depending on the specified output type
            Construct(&'static TypeName),

            /// Extract a value from a composite object
            ///
            /// Takes a single argument (only vector types are supported)
            Extract(u32),

            Call(FunctionRef),
            Parameter(u32, &'static TypeName),
            Return,

            #( #node_variants ),*
        }

        impl Node {
            /// Get the name of this node
            #[inline]
            pub fn to_string(&self) -> &'static str {
                match *self {
                    Node::Input(..) => "Input",
                    Node::Uniform(..) => "Uniform",
                    Node::Output(..) => "Output",
                    Node::Constant(..) => "Constant",
                    Node::Construct(..) => "Construct",
                    Node::Extract(..) => "Extract",
                    Node::Call(..) => "Call",
                    Node::Parameter(..) => "Parameter",
                    Node::Return => "Return",
                    #( #to_string_arms ),*
                }
            }

            /// If possible (the node has no payload), construct a node from its name
            #[inline]
            pub fn from_string(str: &str) -> Option<Self> {
                Some(match str {
                    #( #from_string_arms, )*
                    _ => return None,
                })
            }
        }
    };

    let path_types = Path::new(out_dir).join("node.rs");
    let mut f_node = File::create(&path_types).unwrap();

    write!(f_node, "{}", node).unwrap();
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    types(&out_dir);
    nodes(&out_dir);
}
