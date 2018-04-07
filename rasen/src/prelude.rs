//! Exports everything you probably want to have in scope to get started with Rasen

use std::convert::TryFrom;

pub use module::Module;
pub use graph::*;
pub use builder::*;
pub use types::*;
pub use node::*;

pub use petgraph::graph::NodeIndex;
pub use spirv_headers::ExecutionModel as ShaderType;

use errors::{Result, Error};

/// Transform a node graph to SPIR-V bytecode
pub fn build_program<'a, I>(graph: &'a I, mod_type: ShaderType) -> Result<Vec<u8>> where ModuleBuilder: TryFrom<(&'a I, ShaderType), Error = Error> {
    let program = ModuleBuilder::try_from((graph, mod_type))?;
    program.into_binary()
}

/// Transform a node graph to SPIR-V assembly
pub fn build_program_assembly<'a, I>(graph: &'a I, mod_type: ShaderType) -> Result<String> where ModuleBuilder: TryFrom<(&'a I, ShaderType), Error = Error> {
    let program = ModuleBuilder::try_from((graph, mod_type))?;
    program.into_assembly()
}
