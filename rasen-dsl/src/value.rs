//! Definitions for the Value type

use rasen::prelude::{Graph, NodeIndex};

use std::rc::Rc;
use std::cell::RefCell;
use std::marker::PhantomData;

pub type GraphRef = Rc<RefCell<Graph>>;

/// Representation of a shader value
#[derive(Clone, Debug)]
pub enum Value<T> {
    /// Value backed by actual data
    Concrete(T),
    /// Reference to a node in the graph
    Abstract {
        graph: GraphRef,
        index: NodeIndex<u32>,
        ty: PhantomData<T>,
    },
}

/// Trait implemented by any type the DSL considers the be a "value" (including the Value enum itself)
pub trait IntoValue {
    type Output;
    /// Gets a graph reference from this value, if it holds one
    fn get_graph(&self) -> Option<GraphRef> { None }
    /// Gets the concrete value of this value, if it is indeed concrete
    fn get_concrete(&self) -> Option<Self::Output>;
    /// Registers this value into a Graph and returns the node index
    fn get_index(&self, graph: GraphRef) -> NodeIndex<u32>;
}

impl<T> IntoValue for Value<T> where T: IntoValue + Clone {
    type Output = T;

    fn get_graph(&self) -> Option<GraphRef> {
        match *self {
            Value::Concrete(_) => None,
            Value::Abstract { ref graph, .. } => Some(graph.clone()),
        }
    }

    fn get_concrete(&self) -> Option<T> {
        match *self {
            Value::Concrete(ref v) => Some(v.clone()),
            Value::Abstract { .. } => None,
        }
    }

    fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
        match *self {
            Value::Concrete(ref v) => v.get_index(graph),
            Value::Abstract { index, .. } => index,
        }
    }
}

impl<'a, T> IntoValue for &'a Value<T> where T: IntoValue + Clone {
    type Output = T;

    fn get_graph(&self) -> Option<GraphRef> {
        match **self {
            Value::Concrete(_) => None,
            Value::Abstract { ref graph, .. } => Some(graph.clone()),
        }
    }

    fn get_concrete(&self) -> Option<T> {
        match **self {
            Value::Concrete(ref v) => Some(v.clone()),
            Value::Abstract { .. } => None,
        }
    }

    fn get_index(&self, graph: GraphRef) -> NodeIndex<u32> {
        match **self {
            Value::Concrete(ref v) => v.get_index(graph),
            Value::Abstract { index, .. } => index,
        }
    }
}
