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
pub fn decl_types(out_dir: &str) {
    let path = Path::new(out_dir).join("types.rs");
    let mut file = File::create(&path).unwrap();
    for tokens in types::type_structs() {
        write_tokens(&mut file, tokens);
    }
}

/// Create the declarations of all the GLSL operation functions,
/// and implement the math traits for the GLSL types
pub fn decl_operations(out_dir: &str) {
    let path = Path::new(out_dir).join("operations.rs");
    let mut file = File::create(&path).unwrap();
    for tokens in operations::impl_operations() {
        write_tokens(&mut file, tokens);
    }
    for tokens in math::impl_math() {
        write_tokens(&mut file, tokens);
    }
    for tokens in functions::impl_fn() {
        write_tokens(&mut file, tokens);
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    decl_types(&out_dir);
    decl_operations(&out_dir);
}
