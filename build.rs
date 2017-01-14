#![recursion_limit = "128"]
#![feature(inclusive_range_syntax)]

#[macro_use]
extern crate quote;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const INTS: [(&'static str, &'static str, &'static str); 3] = [
    ("Bool", "b", "bool"),
    ("Int", "i", "i32"),
    ("UInt", "u", "u32"),
];
const FLOATS: [(&'static str, &'static str, &'static str); 2] = [
    ("Float", "", "f32"),
    ("Double", "d", "f64"),
];

fn types(out_dir: &String) {
    let module = quote::Ident::from("$module");
    let constant = quote::Ident::from("$constant");

    let mut const_types = Vec::new();
    let mut from_string_arms = Vec::new();
    let mut typed_variants = Vec::new();
    let mut type_name_arms = Vec::new();
    let mut register_constant_arms = Vec::new();

    for &(name, _, ty) in INTS.iter().chain(FLOATS.iter()) {
        let const_name = quote::Ident::from(name.to_string().to_uppercase());
        let glsl_name = name.to_string().to_lowercase();
        let name = quote::Ident::from(name);

        from_string_arms.push(quote! {
            #glsl_name => TypeName::#const_name,
        });

        type_name_arms.push(quote! {
            &TypedValue::#name(..) => TypeName::#const_name
        });

        let register_val = if ty == "bool" {
            quote! {
                if val {
                    #module.declarations.push(Instruction::ConstantTrue {
                        result_type: TypeId(res_type),
                        result_id: ResultId(res_id),
                    });
                } else {
                    #module.declarations.push(Instruction::ConstantFalse {
                        result_type: TypeId(res_type),
                        result_id: ResultId(res_id),
                    });
                }
            }
        } else {
            let size: usize = if ty == "f64" {2} else {1};
            quote! {
                unsafe {
                    let transmuted: [u32; #size] = ::std::mem::transmute(val);
                    #module.declarations.push(Instruction::Constant {
                        result_type: TypeId(res_type),
                        result_id: ResultId(res_id),
                        val: Box::new(transmuted),
                    });
                }
            }
        };

        register_constant_arms.push(quote! {
            &TypedValue::#name(val) => {
                let res_type = #module.register_type(#constant.to_type_name());
                let res_id = #module.get_id();
                #register_val
                Ok(res_id)
            },
        });

        let ty = quote::Ident::from(ty);
        typed_variants.push(quote! {
            #name(#ty)
        });
    }

    for size in 2u32...4u32 {
        for &(name, prefix, ty) in INTS.iter().chain(FLOATS.iter()) {
            let type_variant = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            let const_name = quote::Ident::from(type_variant.to_uppercase());
            let glsl_name = type_variant.to_lowercase();
            let type_variant = quote::Ident::from(type_variant);

            let const_ty = quote::Ident::from(name.to_string().to_uppercase());
            let name = quote::Ident::from(name);

            const_types.push(quote! {
                pub const #const_name: &'static TypeName = &TypeName::Vec(#size, TypeName::#const_ty);
            });

            from_string_arms.push(quote! {
                #glsl_name => TypeName::#const_name,
            });

            let tuple_fields: Vec<_> = (0..size).map(|_| quote::Ident::from(ty)).collect();
            typed_variants.push(quote! {
                #type_variant( #( #tuple_fields ),* )
            });

            type_name_arms.push(quote! {
                &TypedValue::#type_variant(..) => TypeName::#const_name
            });

            let fields: Vec<_> = (0..size).map(|i| quote::Ident::from(format!("f{}", i))).collect();
            let field_ids: Vec<_> = (0..size).map(|i| quote::Ident::from(format!("f{}_id", i))).collect();

            let register_fields: Vec<_> =
                field_ids.iter().zip(fields.iter())
                    .map(|(id, field)| quote! {
                        let #id = ValueId(#module.register_constant(&TypedValue::#name(#field))?);
                    })
                    .collect();

            register_constant_arms.push(quote! {
                &TypedValue::#type_variant(#( #fields ),*) => {
                    #( #register_fields )*

                    let res_type = #module.register_type(#constant.to_type_name());
                    let res_id = #module.get_id();

                    #module.declarations.push(Instruction::ConstantComposite {
                        result_type: TypeId(res_type),
                        result_id: ResultId(res_id),
                        flds: Box::new([ #( #field_ids ),* ]),
                    });

                    Ok(res_id)
                },
            });
        }

        for &(name, prefix, ty) in FLOATS.iter() {
            let ty = quote::Ident::from(ty);
            let const_ty = quote::Ident::from(name.to_string().to_uppercase());

            let type_variant = format!("{}Mat{}", prefix.to_string().to_uppercase(), size);
            let const_ident = quote::Ident::from(type_variant.to_uppercase());
            let glsl_type = type_variant.to_lowercase();
            let type_variant = quote::Ident::from(type_variant);

            const_types.push(quote! {
                pub const #const_ident: &'static TypeName = &TypeName::Mat(#size, TypeName::#const_ty);
            });

            from_string_arms.push(quote! {
                #glsl_type => TypeName::#const_ident,
            });

            let arr_size = (size * size) as usize;
            typed_variants.push(quote! {
                #type_variant([#ty; #arr_size])
            });

            type_name_arms.push(quote! {
                &TypedValue::#type_variant(..) => TypeName::#const_ident
            });
        }
    }

    let type_name_impl = quote! {
        impl TypeName {
            #( #const_types )*

            #[inline]
            pub fn from_string(ty: &str) -> Option<&'static TypeName> {
                Some(match ty {
                    #( #from_string_arms )*
                    _ => return None
                })
            }
        }
    };

    let typed_value = quote! {
        #[derive(Debug)]
        pub enum TypedValue {
            #( #typed_variants ),*
        }
    };

    let typed_value_impl = quote! {
        impl TypedValue {
            #[inline]
            pub fn to_type_name(&self) -> &'static TypeName {
                match self {
                    #( #type_name_arms ),*
                }
            }
        }
    };

    let path_types = Path::new(out_dir).join("types.rs");
    let mut f_types = File::create(&path_types).unwrap();

    write!(f_types, "{}\n{}\n{}",
        type_name_impl, typed_value, typed_value_impl
    ).unwrap();

    let register_constant = quote! {
        match #constant {
            #( #register_constant_arms )*
            _ => Err(ErrorKind::UnsupportedConstant(#constant.to_type_name()))
        }
    };

    let path_module = Path::new(out_dir).join("module.rs");
    let mut f_module = File::create(&path_module).unwrap();

    write!(f_module,
        "macro_rules! impl_register_constant {{\n( $module:expr, $constant:expr ) => {{\n{}\n}};\n}}",
        register_constant
    ).unwrap();
}

const NODES: [(&'static str, &'static str, &'static str); 23] = [(
    "Normalize",
    "Normalize a vector",
    "Takes a single parameter"
), (
    "Add",
    "Add some values",
    "For the moment, only 2 parameters are supported"
), (
    "Substract",
    "Substract a value from another",
    "Takes 2 parameters"
), (
    "Multiply",
    "Multiply some values",
    "For the moment, only 2 parameters are supported"
), (
    "Divide",
    "Divide a value by another",
    "Takes 2 parameters"
), (
    "Modulus",
    "Compute the modulus of a value by another",
    "Takes 2 parameters"
), (
    "Clamp",
    "Clamp a value in a range",
    "Takes 3 parameters: the value to be clamped, the minimum, and the maximum"
), (
    "Dot",
    "Compute the dot product of 2 vectors",
    "Takes 2 parameters"
), (
    "Cross",
    "Compute the cross product of 2 vectors",
    "Takes 2 parameters"
), (
    "Floor",
    "Round a number to the largest lower or equal integer",
    "Takes a single parameter"
), (
    "Ceil",
    "Round a number to the nearest integer",
    "Takes a single parameter"
), (
    "Round",
    "Round a number to the smallest higher or equal integer",
    "Takes a single parameter"
), (
    "Sin",
    "Compute the sinus of an angle in radians",
    "Takes a single parameter"
), (
    "Cos",
    "Compute the cosinus of an angle in radians",
    "Takes a single parameter"
), (
    "Tan",
    "Compute the tangent of an angle in radians",
    "Takes a single parameter"
), (
    "Pow",
    "Raise a number to a power",
    "Takes 2 parameters"
), (
    "Min",
    "Returns the smallest value of all its arguments",
    "For the moment, only 2 parameters are supported"
), (
    "Max",
    "Return the greatest value of all its arguments",
    "For the moment, only 2 parameters are supported"
), (
    "Length",
    "Computes the length of a vector",
    "Takes a single parameter"
), (
    "Distance",
    "Computes the distance between 2 points",
    "Takes 2 parameters"
), (
    "Reflect",
    "Reflect a vector against a surface normal",
    "Takes 2 parameters"
), (
    "Refract",
    "Computes the refraction of a vector using a surface normal and a refraction indice",
    "Takes 3 parameters"
), (
    "Mix",
    "Computes a linear interpolation between two values",
    "Takes 3 parameters"
)];

fn nodes(out_dir: &String) {
    let mut node_variants = Vec::new();
    let mut to_string_arms = Vec::new();
    let mut from_string_arms = Vec::new();

    for &(name, desc, params) in NODES.iter() {
        let ident = quote::Ident::from(name.to_string());
        node_variants.push(quote! {
            #[doc = #desc]
            #[doc = ""]
            #[doc = #params]
            #ident
        });

        to_string_arms.push(quote! {
            &Node::#ident => #name
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
            Input(u32, &'static TypeName),

            /// Create an output with a location and a type
            ///
            /// Doesn't need to be an output of the graph, but all the outputs should use this type
            Output(u32, &'static TypeName),

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

            #( #node_variants ),*
        }

        impl Node {
            /// Get the name of this node
            #[inline]
            pub fn to_string(&self) -> &'static str {
                match self {
                    &Node::Input(..) => "Input",
                    &Node::Output(..) => "Output",
                    &Node::Constant(..) => "Constant",
                    &Node::Construct(..) => "Construct",
                    &Node::Extract(..) => "Extract",
                    #( #to_string_arms ),*
                }
            }

            /// If possible (the node has no payload), construct a node from it's name
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
