//! Enumerations used in a shader graph

pub use petgraph::graph::NodeIndex;
use petgraph::graph::{
    Externals, Edges
};
use petgraph::{
    Graph as PetGraph, Outgoing, Incoming,
    Directed
};

use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, ValueId
};
use spirv_utils::instruction::*;

use glsl::*;
use super::Module;

/// Define the type of a shader value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TypeName {
    Bool,
    Int,
    Float,
    Vec(u32),
    Mat(u32),
}

/// A typed shader value (used for constants)
#[derive(Debug, Copy, Clone)]
pub enum TypedValue {
    Bool(bool),
    Int(i32),
    Float(f32),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Vec4(f32, f32, f32, f32),
    Mat2([f32; 4]),
    Mat3([f32; 9]),
    Mat4([f32; 16]),
}

impl TypedValue {
    pub fn to_type_name(&self) -> TypeName {
        match *self {
            TypedValue::Bool(_) => TypeName::Bool,
            TypedValue::Int(_) => TypeName::Int,
            TypedValue::Float(_) => TypeName::Float,
            TypedValue::Vec2(_, _) => TypeName::Vec(2),
            TypedValue::Vec3(_, _, _) => TypeName::Vec(3),
            TypedValue::Vec4(_, _, _, _) => TypeName::Vec(4),
            TypedValue::Mat2(_) => TypeName::Mat(2),
            TypedValue::Mat3(_) => TypeName::Mat(3),
            TypedValue::Mat4(_) => TypeName::Mat(4),
        }
    }
}

/// All the supported operations
#[derive(Debug, Copy, Clone)]
pub enum Node {
    /// Create an input with a location and a type
    ///
    /// Incoming values from other nodes are ignored
    Input(u32, TypeName),

    /// Create an output with a location and a type
    ///
    /// Doesn't need to be an output of the graph, but all the outputs should use this type
    Output(u32, TypeName),

    /// Declare a new constant
    ///
    /// Incoming values from other nodes are ignored
    Constant(TypedValue),

    /// Normalize a vector
    ///
    /// Takes a single parameter
    Normalize,

    /// Multiply some values
    ///
    /// For the moment, only 2 parameters are supported
    Multiply,

    /// Clamp a value in a range
    ///
    /// Takes 3 parameters: the value to be clamped, the minimum, and the maximum
    Clamp,

    /// Compute the dot product of 2 vectors
    ///
    /// Takes 2 parameters
    Dot
}

impl Node {
    /// Insert this Node into a Program
    pub fn get_result(&self, module: &mut Module, args: Vec<(TypeName, ResultId)>) -> ResultId {
        use TypeName::*;

        match *self {
            Node::Output(location, ref attr_type) => {
                let type_id = module.register_type(*attr_type);

                let ptr_type = module.get_id().to_type_id();
                module.declarations.push(Instruction::TypePointer {
                    result_type: ptr_type,
                    storage_class: desc::StorageClass::Output,
                    pointee: type_id
                });

                let var_id = module.get_id();
                module.outputs.push(var_id);
                module.declarations.push(Instruction::Variable {
                    result_type: ptr_type,
                    result_id: var_id.to_result_id(),
                    storage_class: desc::StorageClass::Output,
                    init: ValueId(0),
                });

                module.annotations.push(Instruction::Decorate {
                    target: var_id,
                    decoration: Decoration::Location(location)
                });

                module.instructions.push(Instruction::Store {
                    ptr: var_id.to_value_id(),
                    obj: args[0].1.to_value_id(),
                    memory_access: desc::MemoryAccess::empty()
                });

                var_id.to_result_id()
            },
            Node::Input(location, ref attr_type) => {
                let type_id = module.register_type(*attr_type);

                let ptr_type = module.get_id().to_type_id();
                module.declarations.push(Instruction::TypePointer {
                    result_type: ptr_type,
                    storage_class: desc::StorageClass::Input,
                    pointee: type_id
                });

                let var_id = module.get_id();
                module.inputs.push(var_id);
                module.declarations.push(Instruction::Variable {
                    result_type: ptr_type,
                    result_id: var_id.to_result_id(),
                    storage_class: desc::StorageClass::Input,
                    init: ValueId(0),
                });

                module.annotations.push(Instruction::Decorate {
                    target: var_id,
                    decoration: Decoration::Location(location)
                });

                let res_id = module.get_id().to_result_id();
                module.instructions.push(Instruction::Load {
                    result_type: type_id,
                    result_id: res_id,
                    value_id: var_id.to_value_id(),
                    memory_access: desc::MemoryAccess::empty(),
                });

                res_id
            },
            Node::Constant(ref const_type) => {
                module.register_constant(*const_type)
            },
            Node::Normalize => {
                let ext_id = module.import_set(String::from("GLSL.std.450"));

                let float_type = module.register_type(TypeName::Vec(3));

                let args: ::std::vec::Vec<Id> = args.into_iter()
                    .map(|(_, rid)| Id(rid.0))
                    .collect();

                let result_id = module.get_id().to_result_id();
                module.instructions.push(Instruction::ExtInst {
                    result_type: float_type,
                    result_id: result_id,
                    set: ext_id,
                    instruction: GLSL::Normalize as u32,
                    operands: args.into_boxed_slice(),
                });

                result_id
            },
            Node::Multiply => {
                let result_id = module.get_id().to_result_id();

                let (l_type, l_value) = args[0];
                let (r_type, r_value) = args[1];
                let l_value = l_value.to_value_id();
                let r_value = r_value.to_value_id();

                match (l_type, r_type) {
                    (Mat(l_len), Mat(r_len)) if l_len == r_len => {
                        let mat_type = module.register_type(l_type);
                        module.instructions.push(Instruction::MatrixTimesMatrix {
                            result_type: mat_type,
                            result_id: result_id,
                            lhs: l_value,
                            rhs: r_value,
                        });
                    },
                    (Int, Int) => {
                        let float_type = module.register_type(l_type);
                        module.instructions.push(Instruction::IMul {
                            result_type: float_type,
                            result_id: result_id,
                            lhs: l_value,
                            rhs: r_value,
                        });
                    },
                    (Float, Float) => {
                        let float_type = module.register_type(l_type);
                        module.instructions.push(Instruction::FMul {
                            result_type: float_type,
                            result_id: result_id,
                            lhs: r_value,
                            rhs: l_value,
                        });
                    },
                    (Vec(l_len), Vec(r_len)) if l_len == r_len => {
                        let float_type = module.register_type(l_type);
                        module.instructions.push(Instruction::FMul {
                            result_type: float_type,
                            result_id: result_id,
                            lhs: r_value,
                            rhs: l_value,
                        });
                    },
                    (Vec(_), Float) => {
                        let vec_type = module.register_type(l_type);
                        module.instructions.push(Instruction::VectorTimesScalar {
                            result_type: vec_type,
                            result_id: result_id,
                            vector: l_value,
                            scalar: r_value,
                        });
                    },
                    (Float, Vec(_)) => {
                        let vec_type = module.register_type(r_type);
                        module.instructions.push(Instruction::VectorTimesScalar {
                            result_type: vec_type,
                            result_id: result_id,
                            vector: r_value,
                            scalar: l_value,
                        });
                    },
                    (Mat(_), Float) => {
                        let mat_type = module.register_type(l_type);
                        module.instructions.push(Instruction::MatrixTimesScalar {
                            result_type: mat_type,
                            result_id: result_id,
                            matrix: l_value,
                            scalar: r_value,
                        });
                    },
                    (Float, Mat(_)) => {
                        let mat_type = module.register_type(r_type);
                        module.instructions.push(Instruction::MatrixTimesScalar {
                            result_type: mat_type,
                            result_id: result_id,
                            matrix: r_value,
                            scalar: l_value,
                        });
                    },
                    (Vec(v_len), Mat(m_len)) if v_len == m_len => {
                        let vec_type = module.register_type(l_type);
                        module.instructions.push(Instruction::VectorTimesMatrix {
                            result_type: vec_type,
                            result_id: result_id,
                            vector: l_value,
                            matrix: r_value,
                        });
                    },
                    (Mat(m_len), Vec(v_len)) if v_len == m_len => {
                        let vec_type = module.register_type(l_type);
                        module.instructions.push(Instruction::MatrixTimesVector {
                            result_type: vec_type,
                            result_id: result_id,
                            matrix: l_value,
                            vector: r_value,
                        });
                    },
                    _ => panic!("unsupported multiplication")
                }

                result_id
            },
            Node::Clamp => {
                let ext_id = module.import_set(String::from("GLSL.std.450"));

                let float_type = module.register_type(TypeName::Float);

                let args: ::std::vec::Vec<Id> = args.into_iter()
                    .map(|(_, rid)| Id(rid.0))
                    .collect();

                let result_id = module.get_id().to_result_id();
                module.instructions.push(Instruction::ExtInst {
                    result_type: float_type,
                    result_id: result_id,
                    set: ext_id,
                    instruction: GLSL::FClamp as u32,
                    operands: args.into_boxed_slice(),
                });

                result_id
            },
            Node::Dot => {
                let float_type = module.register_type(TypeName::Float);

                let result_id = module.get_id().to_result_id();
                module.instructions.push(Instruction::Dot {
                    result_type: float_type,
                    result_id: result_id,
                    lhs: args[0].1.to_value_id(),
                    rhs: args[1].1.to_value_id(),
                });

                result_id
            },
        }
    }
}

/// Wrapper for the petgraph::Graph struct, with type inference on the edges
pub struct Graph {
    graph: PetGraph<Node, TypeName>
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
    pub fn add_edge(&mut self, from: NodeIndex<u32>, to: NodeIndex<u32>) {
        let type_name = self.infer_type(from);
        self.graph.add_edge(from, to, type_name);
    }

    /// Get a node from the graph
    pub fn node(&self, index: NodeIndex<u32>) -> Node {
        self.graph[index]
    }

    /// List all the outputs of the graph
    pub fn outputs(&self) -> Externals<Node, Directed> {
        self.graph.externals(Outgoing)
    }

    /// List the incoming connections for a node
    pub fn arguments(&self, index: NodeIndex<u32>) -> Edges<TypeName> {
        self.graph.edges_directed(index, Incoming)
    }

    fn infer_type(&self, index: NodeIndex<u32>) -> TypeName {
        let args: Vec<TypeName> = self.graph.neighbors_directed(index, Incoming)
            .map(|index| self.infer_type(index))
            .collect();

        match self.graph[index] {
            Node::Input(_, type_name) => type_name,
            Node::Output(_, type_name) => type_name,
            Node::Constant(value) => value.to_type_name(),
            Node::Normalize => args[0],
            Node::Multiply => args[0],
            Node::Clamp => args[0],
            Node::Dot => TypeName::Float,
        }
    }
}
