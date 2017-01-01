//! Graph building helpers

pub use petgraph::graph::NodeIndex;
use petgraph::{
    Graph as PetGraph, Outgoing, Incoming,
};

pub use super::types::*;
pub use super::node::*;

/// Wrapper for the petgraph::Graph struct, with type inference on the edges
#[derive(Debug)]
pub struct Graph {
    graph: PetGraph<Node, u32>
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Graph {
        Graph {
            graph: PetGraph::new()
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

    /// Get a node from the graph
    pub fn node(&self, index: NodeIndex<u32>) -> Node {
        self.graph[index]
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
    pub fn arguments(&self, index: NodeIndex<u32>) -> Result<Vec<(NodeIndex<u32>, TypeName)>, &'static str> {
        let mut vec: Vec<(NodeIndex<u32>, &u32)> = self.graph.edges_directed(index, Incoming).collect();

        vec.sort_by_key(|&(_, k)| k);

        vec.into_iter()
            .map(|(node, _)| Ok((node, self.infer_type(node)?)))
            .collect()
    }

    fn infer_type(&self, index: NodeIndex<u32>) -> Result<TypeName, &'static str> {
        use types::TypeName::*;

        let args: ::std::vec::Vec<_> = try!(
            self.graph.neighbors_directed(index, Incoming)
                .map(|index| self.infer_type(index))
                .collect()
        );

        Ok(match self.graph[index] {
            Node::Input(_, type_name) => *type_name,
            Node::Output(_, type_name) => *type_name,
            Node::Constant(value) => value.to_type_name(),

            Node::Multiply => {
                if args.len() != 2 {
                    return Err("Not enough arguments to infer return type for Multiply")
                }

                let (l_type, r_type) = (args[0], args[1]);
                match (l_type, r_type) {
                    _ if l_type == r_type && (l_type.is_integer() || l_type.is_float()) => l_type,

                    (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) |
                    (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && (l_scalar.is_integer() || l_scalar.is_float()) => l_type,

                    (Vec(_, l_scalar), _) |
                    (Mat(_, l_scalar), _) if *l_scalar == r_type && r_type.is_float() => l_type,

                    (_, Vec(_, r_scalar)) |
                    (_, Mat(_, r_scalar)) if l_type == *r_scalar && l_type.is_float() => r_type,

                    (Vec(l_len, l_scalar), Mat(r_len, r_scalar)) |
                    (Mat(l_len, l_scalar), Vec(r_len, r_scalar)) |
                    (Mat(l_len, l_scalar), Mat(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && l_scalar.is_float() => l_type,

                    _ => return Err("Unsupported multiplication")
                }
            },

            Node::Sin |
            Node::Cos |
            Node::Tan |
            Node::Dot |
            Node::Length |
            Node::Distance => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer return type")
                }

                match args[0] {
                    Vec(_, scalar) => *scalar,
                    _ => return Err("Invalid (non-vector) argument"),
                }
            },

            Node::Add |
            Node::Substract |
            Node::Normalize |
            Node::Divide |
            Node::Modulus |
            Node::Clamp |
            Node::Mix |
            Node::Cross |
            Node::Pow |
            Node::Min |
            Node::Max |
            Node::Reflect |
            Node::Refract |
            Node::Floor |
            Node::Ceil |
            Node::Round => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer return type")
                }

                args[0]
            },
        })
    }
}
