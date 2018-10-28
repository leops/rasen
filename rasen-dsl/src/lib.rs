//! Exposes a simple DSL for the construction of a data-flow graph for the rasen compiler
//!
//! ```
//! # extern crate rasen;
//! # extern crate rasen_dsl;
//! # use rasen_dsl::prelude::*;
//! # fn main() {
//! let shader = Module::new();
//!
//! let normal: Value<Vec3> = normalize(shader.input(0, "a_normal"));
//! let light = Vec3(0.3, -0.5, 0.2);
//! let color = Vec4(0.25, 0.625, 1.0, 1.0);
//!
//! let res = clamp(dot(normal, light), 0.1f32, 1.0f32) * color;
//! shader.output(0, "o_color", res);
//!
//! # #[allow(unused_variables)]
//! let bytecode = shader.build(ShaderType::Fragment).unwrap();
//! # }
//! ```

#![feature(try_from)]
#![cfg_attr(feature = "functions", feature(fn_traits, unboxed_closures))]
#![warn(clippy::pedantic)]
#![allow(clippy::unseparated_literal_suffix)]

extern crate rasen;

pub mod module;
pub mod operations;
pub mod prelude;
pub mod types;
pub mod value;
