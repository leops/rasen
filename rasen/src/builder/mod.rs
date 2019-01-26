use petgraph::graph::NodeIndex;
use rspirv::mr::Instruction;
use spirv_headers::Word;

use errors::*;
use graph::*;
use module::FunctionRef;
use types::{TypeName, TypedValue};

mod function;
mod module;

pub(crate) use self::module::FunctionData;
pub use self::module::{Builder as ModuleBuilder, Settings};

pub(crate) trait Builder {
    /// Acquire a new identifier to be used in the module
    fn get_id(&mut self) -> Word;

    /// Import an instruction set to the module, returning its ID
    fn import_set(&mut self, name: &'static str) -> Word;

    /// Get the ID corresponding to a Type
    fn register_type(&mut self, type_id: &'static TypeName) -> Word;

    /// Add a new constant to the module, returning its ID
    fn register_constant(&mut self, constant: &TypedValue) -> Result<u32>;

    fn register_uniform(&mut self, location: u32, type_id: &'static TypeName) -> (Word, Word);

    fn push_instruction(&mut self, inst: Instruction);
    fn push_declaration(&mut self, inst: Instruction);
    fn push_output(&mut self, id: Word);
    fn push_input(&mut self, id: Word);
    fn push_annotation(&mut self, inst: Instruction);
    fn push_debug(&mut self, inst: Instruction);
    fn push_function(&mut self, func: FunctionData);

    fn push_parameter(
        &mut self,
        location: Word,
        ty: &'static TypeName,
        inst: Instruction,
    ) -> Result<()>;
    fn set_return(&mut self, ty: &'static TypeName, inst: Instruction) -> Result<()>;

    fn get_result(&self, index: NodeIndex<u32>) -> Option<(&'static TypeName, u32)>;
    fn set_result(&mut self, index: NodeIndex<u32>, res: (&'static TypeName, u32));

    fn get_function(
        &self,
        index: FunctionRef,
    ) -> Option<(Word, &[&'static TypeName], Option<&'static TypeName>)>;

    fn visit(&mut self, graph: &Graph, index: NodeIndex<u32>) -> Result<(&'static TypeName, u32)>
    where
        Self: Sized,
    {
        if let Some(res) = self.get_result(index) {
            return Ok(res);
        }

        let args: Result<Vec<_>> = {
            graph
                .arguments(index)
                .map(|edge| self.visit(graph, edge))
                .collect()
        };

        let node = &graph[index];
        let res = {
            node.get_result(self, args?)
                .chain_err(|| ErrorKind::BuildError(node.to_string(), index.index()))?
        };

        self.set_result(index, res);
        Ok(res)
    }
}
