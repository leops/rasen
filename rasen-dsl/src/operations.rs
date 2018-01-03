//! Exposes Rust counterparts of common GLSL functions

use rasen::prelude::Node;

use types::traits::*;
use types::*;
use value::*;

use std::marker::PhantomData;
use std::ops::{Add, Sub, Mul, Div, Rem};

#[cfg_attr(feature="clippy", allow(needless_pass_by_value))]
pub fn index<T, V, S>(obj: T, index: u32) -> Value<S> where T: IntoValue<Output=V>, V: Vector<S>, S: Scalar {
    if let Some(value) = obj.get_concrete() {
        return value[index].into();
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
    fn iter(obj: &Self) -> Self::Iter {
        let vec: Vec<_> = (0..V::component_count()).map(move |i| index(obj.clone(), i)).collect();
        vec.into_iter()
    }
}

pub fn sample<T, C, V, S>(texture: T, coords: C) -> Value<Vec4> where T: IntoValue<Output=Sampler>, C: IntoValue<Output=V>, V: Vector<S>, S: Scalar {
    if let Some((texture, _)) = texture.get_concrete().and_then(|a| coords.get_concrete().map(|b| (a, b))) {
        return Value::Concrete(texture.0);
    }

    if let Some(graph_ref) = coords.get_graph() {
        let texture = texture.get_index(graph_ref.clone());
        let coords = coords.get_index(graph_ref.clone());

        let index = {
            let mut graph = graph_ref.borrow_mut();
            let index = graph.add_node(Node::Sample);
            graph.add_edge(texture, index, 0);
            graph.add_edge(coords, index, 1);
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

include!(concat!(env!("OUT_DIR"), "/operations.rs"));
