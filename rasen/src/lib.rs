//! Build a SPIR-V module from an operation graph
//!
//! This library lets you define a shader module as a graph (using the `petgraph` library) of
//! `Node`, describing the operations needed to obtain the outputs of the shader.
//!
//! ```
//! # extern crate rasen;
//! # use rasen::prelude::*;
//! # fn main() {
//!     let mut graph = Graph::default();
//!
//!     // A vec3 input at location 0
//!     let normal = graph.add_node(Node::Input(0, TypeName::VEC3));
//!
//!     // Some ambient light constants
//!     let min_light = graph.add_node(Node::Constant(TypedValue::Float(0.1)));
//!     let max_light = graph.add_node(Node::Constant(TypedValue::Float(1.0)));
//!     let light_dir = graph.add_node(Node::Constant(TypedValue::Vec3(0.3, -0.5, 0.2)));
//!
//!     // The Material color (also a constant)
//!     let mat_color = graph.add_node(Node::Constant(TypedValue::Vec4(0.25, 0.625, 1.0, 1.0)));
//!
//!     // Some usual function calls
//!     let normalize = graph.add_node(Node::Normalize);
//!     let dot = graph.add_node(Node::Dot);
//!     let clamp = graph.add_node(Node::Clamp);
//!     let multiply = graph.add_node(Node::Multiply);
//!
//!     // And a vec4 output at location 0
//!     let color = graph.add_node(Node::Output(0, TypeName::VEC4));
//!
//!     // Normalize the normal
//!     graph.add_edge(normal, normalize, 0);
//!
//!     // Compute the dot product of the surface normal and the light direction
//!     graph.add_edge(normalize, dot, 0);
//!     graph.add_edge(light_dir, dot, 1);
//!
//!     // Restrict the result into the ambient light range
//!     graph.add_edge(dot, clamp, 0);
//!     graph.add_edge(min_light, clamp, 1);
//!     graph.add_edge(max_light, clamp, 2);
//!
//!     // Multiply the light intensity by the surface color
//!     graph.add_edge(clamp, multiply, 0);
//!     graph.add_edge(mat_color, multiply, 1);
//!
//!     // Write the result to the output
//!     graph.add_edge(multiply, color, 0);
//!
//! #   #[allow(unused_variables)]
//!     let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
//!     // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
//! # }
//! ```
//!
//! On a lower level, you can use the `Builder` struct to build your module by adding instructions
//! directly into it.
//!

#![feature(box_syntax)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(
    option_unwrap_used, result_unwrap_used,
    single_match_else
))]

extern crate petgraph;
extern crate spirv_headers;
extern crate rspirv;
#[macro_use]
extern crate error_chain;

mod types;
mod operations;
mod builder;
mod node;

pub mod graph;
pub mod errors;
pub mod prelude;
