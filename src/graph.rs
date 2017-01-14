//! Graph building helpers

use petgraph::graph::NodeIndex;
use petgraph::{
    Graph as PetGraph, Outgoing, Incoming, algo,
};

pub use super::types::*;
pub use super::node::*;

/// Wrapper for the petgraph::Graph struct, with type inference on the edges
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

    /// Get a node from the graph
    pub fn node<'a>(&'a self, index: NodeIndex<u32>) -> &'a Node {
        &self.graph[index]
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
    pub fn arguments(&self, index: NodeIndex<u32>) -> Vec<NodeIndex<u32>> {
        let mut vec: Vec<_> = self.graph.edges_directed(index, Incoming).collect();

        vec.sort_by_key(|&(_, w)| w);

        vec.into_iter().map(|(k, _)| k).collect()
    }
}
