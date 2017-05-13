use rasen::prelude::Node;

use types::*;
use operations::*;

use std::ops::Index;
use std::marker::PhantomData;

pub fn index<T, V, R, S>(obj: T, index: u32) -> Value<S> where T: IntoValue<Output=V>, V: Vector<S> + Index<u32, Output=R>, R: Clone + Into<Value<S>>, S: Scalar {
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
