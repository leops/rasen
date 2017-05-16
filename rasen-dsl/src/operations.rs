//! Exposes Rust counterparts of common GLSL functions

use rasen::prelude::Node;

use types::*;
use value::*;

use std::marker::PhantomData;
use std::ops::{Add, Sub, Mul, Div, Rem};

pub fn index<T, V, S>(obj: T, index: u32) -> Value<S> where T: IntoValue<Output=V>, V: Vector<S>, S: Scalar {
    if let Some(value) = obj.get_concrete() {
        return value[index].clone().into();
    }

    if let Some(graph_ref) = obj.get_graph() {
        let source = obj.get_index(graph_ref.clone());
        let index = {
            let mut graph = graph_ref.borrow_mut();
            let index = graph.add_node(Node::Extract(index));
            graph.add_edge(source, index, 0);
            index
        };

        return Value::Abstract {
            graph: graph_ref.clone(),
            index,
            ty: PhantomData,
        };
    }

    unreachable!()
}

impl<V, S> ValueIter<S> for Value<V> where V: Vector<S>, S: Scalar {
    type Iter = ::std::vec::IntoIter<Value<S>>;
    fn iter<'a>(obj: &Self) -> Self::Iter {
        let vec: Vec<_> = (0..V::component_count()).map(move |i| index(obj.clone(), i)).collect();
        vec.into_iter()
    }
}

::rasen_codegen::decl_operations!();
