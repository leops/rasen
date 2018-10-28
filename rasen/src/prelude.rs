//! Exports everything you probably want to have in scope to get started with Rasen

use std::convert::TryFrom;

pub use builder::*;
pub use graph::*;
pub use module::Module;
pub use node::*;
pub use types::*;

pub use petgraph::graph::NodeIndex;
pub use spirv_headers::BuiltIn;
pub use spirv_headers::ExecutionModel as ShaderType;

use errors::{Error, Result};

/// Transform a node graph to SPIR-V bytecode
pub fn build_program<'a, I, S>(graph: &'a I, settings: S) -> Result<Vec<u8>>
where
    ModuleBuilder: TryFrom<(&'a I, S), Error = Error>,
{
    let program = ModuleBuilder::try_from((graph, settings))?;
    program.into_binary()
}

/// Transform a node graph to SPIR-V assembly
pub fn build_program_assembly<'a, I, S>(graph: &'a I, settings: S) -> Result<String>
where
    ModuleBuilder: TryFrom<(&'a I, S), Error = Error>,
{
    let program = ModuleBuilder::try_from((graph, settings))?;
    program.into_assembly()
}
