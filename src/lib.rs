//! Build a SPIR-V module from an operation graph
//!
//! This library lets you define a shader module as a graph (using the `petgraph` library) of
//! `Node`, describing the operations needed to obtain the outputs of the shader.
//!
//! ```
//! extern crate rasen;
//!
//! use rasen::*;
//!
//! fn main() {
//!     let mut graph = Graph::new();
//!
//!     // A vec3 input at location 0
//!     let normal = graph.add_node(Node::Input(0, TypeName::Vec(3)));
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
//!     let color = graph.add_node(Node::Output(0, TypeName::Vec(4)));
//!
//!     // Normalize the normal
//!     graph.add_edge(normal, normalize);
//!
//!     // Compute the dot product of the surface normal and the light direction
//!     graph.add_edge(normalize, dot);
//!     graph.add_edge(light_dir, dot);
//!
//!     // Restrict the result into the ambient light range
//!     graph.add_edge(dot, clamp);
//!     graph.add_edge(min_light, clamp);
//!     graph.add_edge(max_light, clamp);
//!
//!     // Multiply the light intensity by the surface color
//!     graph.add_edge(clamp, multiply);
//!     graph.add_edge(mat_color, multiply);
//!
//!     // Write the result to the output
//!     graph.add_edge(multiply, color);
//!
//!     let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
//!     // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
//! }
//! ```
//!
//! On a lower level, you can use the `Module` struct to build your module by adding instructions
//! directly into it.
//!

extern crate petgraph;
extern crate spirv_utils;

pub mod graph;
pub mod glsl;
mod module;

pub use module::*;
pub use graph::*;

/// Transform a node graph to SPIR-V bytecode
pub fn build_program(graph: &Graph, mod_type: ShaderType) -> Result<Vec<u8>, &'static str> {
    let program = try!(Module::build(graph, mod_type));
    Ok(program.get_bytecode())
}
