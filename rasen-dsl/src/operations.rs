//! Exposes Rust counterparts of common GLSL functions

use rasen::prelude::Node;

use types::traits::*;
use types::*;
use value::*;
#[cfg(feature = "functions")]
use module::*;

use std::vec::IntoIter;
use std::marker::PhantomData;
use std::ops::{Add, Sub, Mul, Div, Rem};

#[cfg_attr(feature="clippy", allow(needless_pass_by_value))]
pub fn index<T, V, S>(obj: T, index: u32) -> Value<S> where T: IntoValue<Output=V>, V: Vector<S>, S: Scalar {
    match obj.into_value() {
        Value::Concrete(value) => {
            value[index].into()
        },
        Value::Abstract { module, function, index: source, .. } => {
            let index = {
                let module = module.borrow_mut();
                let mut graph = function.get_graph_mut(module);
                let index = graph.add_node(Node::Extract(index));
                graph.add_edge(source, index, 0);
                index
            };

            return Value::Abstract {
                module,
                function,
                index,
                ty: PhantomData,
            };
        },
    }
}

impl<V, S> ValueIter<S> for Value<V> where V: Vector<S>, S: Scalar {
    type Iter = IntoIter<Value<S>>;
    fn iter(obj: &Self) -> Self::Iter {
        let vec: Vec<_> = (0..V::component_count()).map(move |i| index(obj, i)).collect();
        vec.into_iter()
    }
}

pub fn sample<T, C, V, S>(texture: T, coords: C) -> Value<Vec4> where T: IntoValue<Output=Sampler>, C: IntoValue<Output=V>, V: Vector<S>, S: Scalar {
    let (module, function, tex, coords) = match (texture.into_value(), coords.into_value()) {
        (Value::Concrete(texture), Value::Concrete(_)) => {
            return Value::Concrete(texture.0);
        },

        (Value::Abstract { module, function, index: tex, .. }, coords @ Value::Concrete(_)) => {
            let coords = {
                let module = module.borrow_mut();
                let graph = function.get_graph_mut(module);
                coords.get_index(graph)
            };

            (module, function, tex, coords)
        },

        (tex @ Value::Concrete(_), Value::Abstract { module, function, index: coords, .. }) => {
            let tex = {
                let module = module.borrow_mut();
                let graph = function.get_graph_mut(module);
                tex.get_index(graph)
            };

            (module, function, tex, coords)
        },

        (Value::Abstract { module, function, index: tex, .. }, Value::Abstract { index: coords, .. }) => {
            (module, function, tex, coords)
        },
    };

    let index = {
        let module = module.borrow_mut();
        let mut graph = function.get_graph_mut(module);

        let index = graph.add_node(Node::Sample);
        graph.add_edge(tex, index, 0);
        graph.add_edge(coords, index, 1);
        index
    };

    Value::Abstract {
        module,
        function,
        index,
        ty: PhantomData,
    }
}

include!(concat!(env!("OUT_DIR"), "/operations.rs"));
