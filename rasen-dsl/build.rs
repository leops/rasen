//! Internal procedural macro provider for the rasen-dsl crate

#![recursion_limit = "256"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate syn;
#[macro_use] extern crate quote;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod codegen;
use codegen::*;

/// Create the declarations of all the GLSL type structs
pub fn decl_types(out_dir: &str) {
    let path = Path::new(out_dir).join("types.rs");
    let mut file = File::create(&path).unwrap();
    for tokens in types::type_structs() {
        writeln!(file, "{}", tokens).unwrap();
    }
}

/// Create the declarations of all the GLSL operation functions,
/// and implement the math traits for the GLSL types
pub fn decl_operations(out_dir: &str) {
    let path = Path::new(out_dir).join("operations.rs");
    let mut file = File::create(&path).unwrap();
    for tokens in operations::impl_operations() {
        writeln!(file, "{}", tokens).unwrap();
    }
    for tokens in math::impl_math() {
        writeln!(file, "{}", tokens).unwrap();
    }
    for tokens in functions::impl_fn() {
        writeln!(file, "{}", tokens).unwrap();
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    decl_types(&out_dir);
    decl_operations(&out_dir);
}
