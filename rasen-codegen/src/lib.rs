//! Internal procedural macro provider for the rasen-dsl crate

#![recursion_limit = "256"]
#![feature(proc_macro, inclusive_range_syntax, box_syntax)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

mod defs;
mod types;
mod operations;
mod mul;

use proc_macro::TokenStream;

/// Create the declarations of all the GLSL type structs
#[proc_macro]
pub fn decl_types(_: TokenStream) -> TokenStream {
    let types = types::type_structs();
    let gen = quote! { #( #types )* };
    gen.parse().unwrap()
}

/// Create the declarations of all the GLSL operation functions
#[proc_macro]
pub fn decl_operations(_: TokenStream) -> TokenStream {
    let ops = operations::impl_operations();
    let gen = quote! { #( #ops )* };
    gen.parse().unwrap()
}

/// Implement the Mul trait on all the declared types
#[proc_macro]
pub fn impl_mul(_: TokenStream) -> TokenStream {
    let impls = mul::impl_mul();
    let gen = quote! { #( #impls )* };
    gen.parse().unwrap()
}
