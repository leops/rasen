//! Exports everything you probably want to have in scope to get started with Rasen

pub use graph::*;
pub use builder::*;
pub use types::*;
pub use node::*;

pub use petgraph::graph::NodeIndex;
pub use spirv_headers::ExecutionModel as ShaderType;

use errors::Result;

/// Transform a node graph to SPIR-V bytecode
pub fn build_program(graph: &Graph, mod_type: ShaderType) -> Result<Vec<u8>> {
    let program = Builder::from_graph(graph, mod_type)?;
    program.into_binary()
}

/// Transform a node graph to SPIR-V assembly
pub fn build_program_assembly(graph: &Graph, mod_type: ShaderType) -> Result<String> {
    let program = Builder::from_graph(graph, mod_type)?;
    program.into_assembly()
}
