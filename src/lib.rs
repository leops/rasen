//! Build a SPIR-V module from an operation graph
//!
//! This library lets you define a shader module as a graph (using the `petgraph` library) of
//! `Node`, describing the operations needed to obtain the outputs of the shader.
//!
//! ```
//! #[macro_use]
//! extern crate rasen;
//!
//! use rasen::*;
//! use rasen::TypedValue::*;
//!
//! fn main() {
//!     let graph = rasen_graph! {
//!         // The only output of this graph is a vec4, at location 0
//!         Output(0, TypeName::VEC4) {
//!             // Multiply the light intensity by the surface color
//!             Multiply {
//!                 // Restrict the intensity into the ambient light range
//!                 Clamp {
//!                     // Compute the dot product of the surface normal and the light direction
//!                     Dot {
//!                         // Normalize the normal
//!                         Normalize {
//!                             // The surface normal, a vec3 input at location 0
//!                             Input(0, TypeName::VEC3)
//!                         }
//!                         // The directional light direction
//!                         Constant(Vec3(0.3, -0.5, 0.2))
//!                     }
//!                     // The minimum / maximum light levels
//!                     Constant(Float(0.1))
//!                     Constant(Float(1.0))
//!                 }
//!                 // The Material color
//!                 Constant(Vec4(0.25, 0.625, 1.0, 1.0))
//!             }
//!         };
//!     };
//!
//!     let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
//!     // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
//! }
//! ```
//!
//! On a lower level, you can use the `Builder` struct to build your module by adding instructions
//! directly into it.
//!

#![feature(associated_consts, conservative_impl_trait)]

extern crate petgraph;
extern crate spirv_headers;
extern crate rspirv;
#[macro_use]
extern crate error_chain;

pub mod graph;
pub mod glsl;
pub mod errors;

mod types;
mod operations;
mod builder;
mod node;
mod macros;

use errors::*;
pub use graph::*;
pub use builder::*;

/// Transform a node graph to SPIR-V bytecode
pub fn build_program(graph: &Graph, mod_type: ShaderType) -> Result<Vec<u8>> {
    let program = Builder::build(graph, mod_type)?;
    Ok(program.into_bytecode())
}
