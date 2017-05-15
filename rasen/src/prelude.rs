//! Exports everything you probably want to have in scope to get started with Rasen

pub use glsl;
pub use graph::*;
pub use builder::*;
pub use types::*;
pub use node::*;

pub use petgraph::graph::NodeIndex;
pub use spirv_headers::ExecutionModel as ShaderType;

use errors::Result;

/// Transform a node graph to SPIR-V bytecode
pub fn build_program(graph: &Graph, mod_type: ShaderType) -> Result<Vec<u8>> {
    let program = Builder::build(graph, mod_type)?;
    program.into_bytecode()
}