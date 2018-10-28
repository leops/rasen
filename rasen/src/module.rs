//! A Module is the advanced entry point for the Rasen compiler. It holds the graph for the main function,
//! as well as the subgraphs for all the user-defined functions

use graph::Graph;
use std::ops::{Index, IndexMut};

/// An opaque pointer struct to a function
#[derive(Copy, Clone, Debug)]
pub struct FunctionRef(pub(crate) usize);

/// A container for complex shader programs with multiple functions
#[derive(Debug, Default)]
pub struct Module {
    pub main: Graph,
    pub(crate) functions: Vec<Graph>,
}

impl Module {
    /// Add a function to the graph
    pub fn add_function(&mut self) -> FunctionRef {
        let index = self.functions.len();
        self.functions.push(Graph::default());
        FunctionRef(index)
    }

    /// Get a reference to a function's graph from its index
    pub fn function(&mut self, index: FunctionRef) -> Option<&mut Graph> {
        self.functions.get_mut(index.0)
    }
}

impl Index<FunctionRef> for Module {
    type Output = Graph;

    fn index(&self, index: FunctionRef) -> &Graph {
        &self.functions[index.0]
    }
}

impl IndexMut<FunctionRef> for Module {
    fn index_mut(&mut self, index: FunctionRef) -> &mut Graph {
        &mut self.functions[index.0]
    }
}
