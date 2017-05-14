use rasen::prelude::{Node, Graph, NodeIndex};

use types::*;

mod index;
pub use self::index::*;

mod mul;
pub use self::mul::*;

use std::rc::Rc;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::iter::Sum;
use std::ops::{Mul, Div, Index};

pub type GraphRef = Rc<RefCell<Graph>>;

/// Value trait
#[derive(Clone, Debug)]
pub enum Value<T> {
    Concrete(T),
    Abstract {
        graph: GraphRef,
        index: NodeIndex<u32>,
        ty: PhantomData<T>,
    },
}

pub trait IntoValue {
    type Output;
    fn get_graph(&self) -> Option<GraphRef>;
    fn get_concrete(&self) -> Option<Self::Output>;
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

/// Generic sqrt operation
pub trait Sqrt {
    fn sqrt(val: Self) -> Self;
}

impl Sqrt for f32 {
    fn sqrt(val: Self) -> Self {
        val.sqrt()
    }
}
impl Sqrt for f64 {
    fn sqrt(val: Self) -> Self {
        val.sqrt()
    }
}

::rasen_codegen::decl_operations!();
