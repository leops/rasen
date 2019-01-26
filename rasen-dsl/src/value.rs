//! Definitions for the Value type

use rasen::{
    module::FunctionRef,
    prelude::{Graph, NodeIndex},
};

use std::{cell::RefMut, marker::PhantomData};

use module::{Module, ModuleRef};

pub(crate) type GraphRef<'a> = RefMut<'a, Graph>;

#[doc(hidden)]
#[derive(Copy, Clone, Debug)]
pub enum FuncKind {
    Main,
    Ref(FunctionRef),
}

impl FuncKind {
    pub fn get_graph_mut<'a>(&self, module: ModuleRef<'a>) -> GraphRef<'a> {
        match *self {
            FuncKind::Main => RefMut::map(module, |module| {
                // let module = shader.module.get_mut();
                &mut module.main
            }),
            FuncKind::Ref(index) => RefMut::map(module, |module| {
                // let module = shader.module.get_mut();
                &mut module[index]
            }),
        }
    }
}

/// Representation of a shader value
#[derive(Clone, Debug)]
pub enum Value<T> {
    /// Value backed by actual data
    Concrete(T),
    /// Reference to a node in the graph
    Abstract {
        module: Module,
        function: FuncKind,
        index: NodeIndex<u32>,
        ty: PhantomData<T>,
    },
}

impl<T> Value<T> {
    #[doc(hidden)]
    pub fn get_module(&self) -> Option<Module> {
        match *self {
            Value::Concrete(_) => None,
            Value::Abstract { ref module, .. } => Some(module.clone()),
        }
    }
}

/// Trait implemented by any type the DSL considers the be a "value" (including the Value enum itself)
#[allow(clippy::module_name_repetitions)]
pub trait IntoValue {
    type Output;

    // Convert this object into a Value object
    fn into_value(self) -> Value<Self::Output>;
    /// Registers this value into a Graph and returns the node index
    fn get_index(&self, graph: GraphRef) -> NodeIndex<u32>;
}

impl<T> IntoValue for Value<T>
where
    T: IntoValue + Clone,
{
    type Output = T;

    fn into_value(self) -> Self {
        self
    }

    fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
        match *self {
            Value::Concrete(ref v) => v.get_index(graph),
            Value::Abstract { index, .. } => index,
        }
    }
}

impl<'a, T> IntoValue for &'a Value<T>
where
    T: IntoValue + Clone,
{
    type Output = T;

    fn into_value(self) -> Value<T> {
        self.clone()
    }

    fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
        match **self {
            Value::Concrete(ref v) => v.get_index(graph),
            Value::Abstract { index, .. } => index,
        }
    }
}
