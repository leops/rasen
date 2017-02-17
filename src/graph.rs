//! Graph building helpers

use std::ops::Index;

use petgraph::graph::NodeIndex;
use petgraph::{
    Graph as PetGraph, Outgoing, Incoming, algo,
};

pub use super::types::*;
pub use super::node::*;

/// Convenience wrapper for `petgraph::Graph`
#[derive(Debug)]
pub struct Graph {
    graph: PetGraph<Node, u32>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Graph {
        Graph {
            graph: PetGraph::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> NodeIndex<u32> {
        self.graph.add_node(node)
    }

    /// Add an edge between two nodes in the graph, infering the result type of the origin node
    pub fn add_edge(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>, index: u32) {
        self.graph.add_edge(from, to, index);
    }

    pub fn has_cycle(&self) -> bool {
        algo::is_cyclic_directed(&self.graph)
    }

    /// List all the outputs of the graph
    pub fn outputs<'a>(&'a self) -> Box<Iterator<Item=NodeIndex<u32>> + 'a> {
        Box::new(
            self.graph.externals(Outgoing)
                .filter(move |index| match self.graph[*index] {
                    Node::Output(_, _) => true,
                    _ => false,
                })
        )
    }

    /// List the incoming connections for a node
    pub fn arguments<'a>(&'a self, index: NodeIndex<u32>) -> Box<Iterator<Item=NodeIndex<u32>> + 'a> {
        let mut vec: Vec<_> = self.graph.edges_directed(index, Incoming).collect();

        vec.sort_by_key(|&(_, w)| w);

        Box::new(
            vec.into_iter().map(|(k, _)| k)
        )
    }
}

impl Index<NodeIndex<u32>> for Graph {
    type Output = Node;

    /// Get a node from the graph
    fn index(&self, index: NodeIndex<u32>) -> &Node {
        &self.graph[index]
    }
}
