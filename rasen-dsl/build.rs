//! Internal procedural macro provider for the rasen-dsl crate

#![recursion_limit = "256"]
#![warn(clippy::all, clippy::pedantic)]

extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use std::{env, fmt::Write as FmtWrite, fs::File, io::Write, path::Path};

mod codegen;
use codegen::*;

struct Files {
    types: Vec<TokenStream>,
    operations: Vec<TokenStream>,
    container: Vec<TokenStream>,
    context: Vec<TokenStream>,
    parse: Vec<TokenStream>,
    execute: Vec<TokenStream>,
    module: Vec<TokenStream>,
}

impl Files {
    fn create() -> Self {
        Files {
            types: Vec::new(),
            operations: Vec::new(),
            container: Vec::new(),
            context: Vec::new(),
            parse: Vec::new(),
            execute: Vec::new(),
            module: Vec::new(),
        }
    }

    fn write(self) {
        let out_dir = env::var("OUT_DIR").unwrap();

        {
            let path = Path::new(&out_dir).join("types.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.types;
            write_tokens(&mut file, quote! { #( #tokens )* });
        }

        {
            let path = Path::new(&out_dir).join("operations.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.operations;
            write_tokens(&mut file, quote! { #( #tokens )* });
        }

        {
            let path = Path::new(&out_dir).join("container.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.container;
            write_tokens(&mut file, quote! {
                pub trait Container<T> {
                    type Value: Copy;
                    #( #tokens )*

                    fn sample(sampler: Value<Self, Sampler>, uv: Value<Self, Vec2>) -> Value<Self, Vec4>
                        where Self: Container<Sampler> + Container<Vec2> + Container<Vec4>;
                }
            });
        }

        {
            let path = Path::new(&out_dir).join("context.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.context;
            write_tokens(&mut file, quote! {
                pub trait Context: Container<Sampler> + #( #tokens )+* {}
            });
        }

        {
            let path = Path::new(&out_dir).join("parse.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.parse;
            write_tokens(&mut file, quote! {
                impl<T> Container<T> for Parse {
                    type Value = ParseNode;
                    #( #tokens )*

                    fn sample(sampler: Value<Self, Sampler>, uv: Value<Self, Vec2>) -> Value<Self, Vec4> {
                        with_graph(|graph| {
                            let node = graph.add_node(Node::Sample);
                            graph.add_edge(sampler.0, node, 0);
                            graph.add_edge(uv.0, node, 1);
                            Value(node)
                        })
                    }
                }
            });
        }

        {
            let path = Path::new(&out_dir).join("execute.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.execute;
            write_tokens(&mut file, quote! {
                impl<T: Copy> Container<T> for Execute {
                    type Value = T;
                    #( #tokens )*
                    
                    #[inline]
                    fn sample(sampler: Value<Self, Sampler>, _uv: Value<Self, Vec2>) -> Value<Self, Vec4> {
                        Value((sampler.0).0)
                    }
                }
            });
        }

        {
            let path = Path::new(&out_dir).join("module.rs");
            let mut file = File::create(&path).unwrap();

            let tokens = self.module;
            write_tokens(&mut file, quote! { #( #tokens )* });
        }
    }
}

fn write_tokens(file: &mut File, tokens: TokenStream) {
    let mut line = String::new();
    write!(line, "{}", tokens).unwrap();

    writeln!(file, "{}", {
        line.chars()
            .flat_map(|chr| match chr {
                '{' => vec!['{', '\n'],
                ';' => vec![';', '\n'],
                '}' => vec!['\n', '}'],
                any => vec![any],
            })
            .collect::<String>()
    })
    .unwrap();
}

/// Create the declarations of all the GLSL type structs
fn decl_types(files: &mut Files) {
    for tokens in types::type_structs() {
        let [types, container, context, parse, execute] = tokens;
        files.types.push(types);
        files.container.push(container);
        files.context.push(context);
        files.parse.push(parse);
        files.execute.push(execute);
    }
}

/// Create the declarations of all the GLSL operation functions,
/// and implement the math traits for the GLSL types
fn decl_operations(files: &mut Files) {
    for tokens in operations::impl_operations() {
        let [container, parse, execute, operations] = tokens;
        files.container.push(container);
        files.parse.push(parse);
        files.execute.push(execute);
        files.operations.push(operations);
    }
    for tokens in math::impl_math() {
        let (container, parse, execute, types) = tokens;
        files.types.extend(types);
        files.container.push(container);
        files.parse.push(parse);
        files.execute.push(execute);
    }
    for tokens in functions::impl_fn() {
        files.module.push(tokens);
    }
}

fn main() {
    let mut files = Files::create();
    decl_types(&mut files);
    decl_operations(&mut files);
    files.write();
}
