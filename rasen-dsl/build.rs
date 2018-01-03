//! Internal procedural macro provider for the rasen-dsl crate

#![recursion_limit = "256"]
#![feature(proc_macro, inclusive_range_syntax, box_syntax)]
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
    let types = types::type_structs();
    let gen = quote! { #( #types )* };

    let path = Path::new(out_dir).join("types.rs");
    let mut file = File::create(&path).unwrap();
    write!(file, "{}", gen).unwrap();
}

/// Create the declarations of all the GLSL operation functions,
/// and implement the math traits for the GLSL types
pub fn decl_operations(out_dir: &str) {
    let ops = operations::impl_operations();
    let math = math::impl_math();
    let gen = quote! {
        #( #ops )*
        #( #math )*
    };

    let path = Path::new(out_dir).join("operations.rs");
    let mut file = File::create(&path).unwrap();
    write!(file, "{}", gen).unwrap();
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    decl_types(&out_dir);
    decl_operations(&out_dir);
}
