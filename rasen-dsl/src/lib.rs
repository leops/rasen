//! Exposes a simple DSL for the construction of a data-flow graph for the rasen compiler
//!
//! ```
//! # extern crate rasen;
//! # extern crate rasen_dsl;
//! # use rasen_dsl::prelude::*;
//! # fn main() {
//! let shader = Module::build(|module| {
//!     let normal = normalize(module.input(0, "a_normal"));
//!     let light = vec3(0.3f32, -0.5f32, 0.2f32);
//!     let color = vec4(0.25f32, 0.625f32, 1.0f32, 1.0f32);
//!    
//!     let res = clamp(dot(normal, light), 0.1f32, 1.0f32) * color;
//!     module.output(0, "o_color", res);
//! });
//! 
//! # #[allow(unused_variables)]
//! let bytecode = build_program(&shader, ShaderType::Fragment).unwrap();
//! # }
//! ```

#![feature(fn_traits, unboxed_closures)]
#![warn(clippy::all, clippy::pedantic)]

extern crate rasen;

pub mod context;
pub mod module;
pub mod operations;
pub mod prelude;
pub mod types;
pub mod value;
