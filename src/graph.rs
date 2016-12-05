//! Enumerations used in a shader graph

pub use petgraph::graph::NodeIndex;
use petgraph::{
    Graph as PetGraph, Outgoing, Incoming,
};

use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId,
};
use spirv_utils::instruction::*;

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

    /// Add some values
    ///
    /// For the moment, only 2 parameters are supported
    Add,

    /// Substract a value from another
    ///
    /// Takes 2 parameters
    Substract,

    /// Multiply some values
    ///
    /// For the moment, only 2 parameters are supported
    Multiply,

    /// Divide a value by another
    ///
    /// Takes 2 parameters
    Divide,

    /// Compute the modulus of a value by another
    ///
    /// Takes 2 parameters
    Modulus,

    /// Clamp a value in a range
    ///
    /// Takes 3 parameters: the value to be clamped, the minimum, and the maximum
    Clamp,

    /// Compute the dot product of 2 vectors
    ///
    /// Takes 2 parameters
    Dot,

    /// Compute the cross product of 2 vectors
    ///
    /// Takes 2 parameters
    Cross,

    /// Round a number to the largest lower or equal integer
    ///
    /// Takes a single parameter
    Floor,

    /// Round a number to the nearest integer
    ///
    /// Takes a single parameter
    Ceil,

    /// Round a number to the smallest higher or equal integer
    ///
    /// Takes a single parameter
    Round,

    /// Compute the sinus of an angle in radians
    ///
    /// Takes a single parameter
    Sin,

    /// Compute the cosinus of an angle in radians
    ///
    /// Takes a single parameter
    Cos,

    /// Compute the tangent of an angle in radians
    ///
    /// Takes a single parameter
    Tan,

    /// Raise a number to a power
    ///
    /// Takes 2 parameters
    Pow,

    /// Returns the smallest value of all its arguments
    ///
    /// For the moment, only 2 parameters are supported
    Min,

    /// Return the greatest value of all its arguments
    ///
    /// For the moment, only 2 parameters are supported
    Max,

    /// Computes the length of a vector
    ///
    /// Takes a single parameter
    Length,

    /// Computes the distance between 2 points
    ///
    /// Takes 2 parameters
    Distance,

    /// Reflect a vector against a surface normal
    ///
    /// Takes 2 parameters
    Reflect,

    /// Computes the refraction of a vector using a surface normal and a refraction indice
    ///
    /// Takes 3 parameters
    Refract,
}

macro_rules! impl_math_op {
    ( $args:ident, $module:ident, $name:expr, $iopcode:ident, $fopcode:ident ) => {
        {
            if $args.len() != 2 {
                return Err(concat!("Wrong number of arguments for ", $name));
            }

            let result_id = $module.get_id();

            let (l_type, l_value) = $args[0];
            let (r_type, r_value) = $args[1];

            match (l_type, r_type) {
                (Int, Int) => {
                    let int_type = $module.register_type(l_type);

                    $module.instructions.push(Instruction::$iopcode {
                        result_type: TypeId(int_type),
                        result_id: ResultId(result_id),
                        lhs: ValueId(l_value),
                        rhs: ValueId(r_value),
                    });
                },
                (Float, Float) => {
                    let float_type = $module.register_type(l_type);

                    $module.instructions.push(Instruction::$fopcode {
                        result_type: TypeId(float_type),
                        result_id: ResultId(result_id),
                        lhs: ValueId(r_value),
                        rhs: ValueId(l_value),
                    });
                },
                _ => return Err("Unsupported operation")
            }

            result_id
        }
    };
}

macro_rules! impl_glsl_call {
    ( $args:ident, $module:ident, $function:expr, $argc:expr, $result:expr ) => {
        {
            if $args.len() != $argc {
                return Err(concat!("Wrong number of arguments for ", stringify!($function)));
            }

            let ext_id = $module.import_set(String::from("GLSL.std.450"));

            let float_type = $module.register_type($result);

            let args: ::std::vec::Vec<_> = $args.into_iter()
                .map(|(_, rid)| Id(rid))
                .collect();

            let result_id = $module.get_id();

            $module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(float_type),
                result_id: ResultId(result_id),
                set: ValueId(ext_id),
                instruction: $function as u32,
                operands: args.into_boxed_slice(),
            });

            result_id
        }
    };
}

impl Node {
    /// Insert this Node into a Program
    pub fn get_result(&self, module: &mut Module, args: Vec<(TypeName, u32)>) -> Result<u32, &'static str> {
        use TypeName::*;
        use glsl::GLSL::*;

        Ok(match *self {
            Node::Output(location, ref attr_type) => {
                if args.len() != 1 {
                    return Err("Wrong number of arguments for Output");
                }

                let type_id = module.register_type(*attr_type);

                let ptr_type = module.get_id();

                module.declarations.push(Instruction::TypePointer {
                    result_type: TypeId(ptr_type),
                    storage_class: desc::StorageClass::Output,
                    pointee: TypeId(type_id)
                });

                let var_id = module.get_id();

                module.outputs.push(Id(var_id));

                module.declarations.push(Instruction::Variable {
                    result_type: TypeId(ptr_type),
                    result_id: ResultId(var_id),
                    storage_class: desc::StorageClass::Output,
                    init: ValueId(0),
                });

                module.annotations.push(Instruction::Decorate {
                    target: Id(var_id),
                    decoration: Decoration::Location(location)
                });

                module.instructions.push(Instruction::Store {
                    ptr: ValueId(var_id),
                    obj: ValueId(args[0].1),
                    memory_access: desc::MemoryAccess::empty()
                });

                var_id
            },

            Node::Input(location, ref attr_type) => {
                let type_id = module.register_type(*attr_type);

                let ptr_type = module.get_id();

                module.declarations.push(Instruction::TypePointer {
                    result_type: TypeId(ptr_type),
                    storage_class: desc::StorageClass::Input,
                    pointee: TypeId(type_id)
                });

                let var_id = module.get_id();

                module.inputs.push(Id(var_id));

                module.declarations.push(Instruction::Variable {
                    result_type: TypeId(ptr_type),
                    result_id: ResultId(var_id),
                    storage_class: desc::StorageClass::Input,
                    init: ValueId(0),
                });

                module.annotations.push(Instruction::Decorate {
                    target: Id(var_id),
                    decoration: Decoration::Location(location)
                });

                let res_id = module.get_id();

                module.instructions.push(Instruction::Load {
                    result_type: TypeId(type_id),
                    result_id: ResultId(res_id),
                    value_id: ValueId(var_id),
                    memory_access: desc::MemoryAccess::empty(),
                });

                res_id
            },

            Node::Constant(ref const_type) => {
                try!(module.register_constant(*const_type))
            },

            Node::Normalize => impl_glsl_call!(
                args, module,
                Normalize,
                1, TypeName::Vec(3)
            ),

            Node::Add => impl_math_op!(
                args, module, "Add",
                IAdd, FAdd
            ),

            Node::Substract => impl_math_op!(
                args, module, "Substract",
                ISub, FSub
            ),

            Node::Multiply => {
                if args.len() != 2 {
                    return Err("Wrong number of arguments for Multiply");
                }

                let result_id = module.get_id();

                let (l_type, l_value) = args[0];
                let (r_type, r_value) = args[1];
                let l_value = l_value;
                let r_value = r_value;

                match (l_type, r_type) {
                    (Mat(l_len), Mat(r_len)) if l_len == r_len => {
                        let mat_type = module.register_type(l_type);

                        module.instructions.push(Instruction::MatrixTimesMatrix {
                            result_type: TypeId(mat_type),
                            result_id: ResultId(result_id),
                            lhs: ValueId(l_value),
                            rhs: ValueId(r_value),
                        });
                    },
                    (Int, Int) => {
                        let float_type = module.register_type(l_type);

                        module.instructions.push(Instruction::IMul {
                            result_type: TypeId(float_type),
                            result_id: ResultId(result_id),
                            lhs: ValueId(l_value),
                            rhs: ValueId(r_value),
                        });
                    },
                    (Float, Float) => {
                        let float_type = module.register_type(l_type);

                        module.instructions.push(Instruction::FMul {
                            result_type: TypeId(float_type),
                            result_id: ResultId(result_id),
                            lhs: ValueId(r_value),
                            rhs: ValueId(l_value),
                        });
                    },
                    (Vec(l_len), Vec(r_len)) if l_len == r_len => {
                        let float_type = module.register_type(l_type);

                        module.instructions.push(Instruction::FMul {
                            result_type: TypeId(float_type),
                            result_id: ResultId(result_id),
                            lhs: ValueId(r_value),
                            rhs: ValueId(l_value),
                        });
                    },
                    (Vec(_), Float) => {
                        let vec_type = module.register_type(l_type);

                        module.instructions.push(Instruction::VectorTimesScalar {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            vector: ValueId(l_value),
                            scalar: ValueId(r_value),
                        });
                    },
                    (Float, Vec(_)) => {
                        let vec_type = module.register_type(r_type);

                        module.instructions.push(Instruction::VectorTimesScalar {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            vector: ValueId(r_value),
                            scalar: ValueId(l_value),
                        });
                    },
                    (Mat(_), Float) => {
                        let mat_type = module.register_type(l_type);

                        module.instructions.push(Instruction::MatrixTimesScalar {
                            result_type: TypeId(mat_type),
                            result_id: ResultId(result_id),
                            matrix: ValueId(l_value),
                            scalar: ValueId(r_value),
                        });
                    },
                    (Float, Mat(_)) => {
                        let mat_type = module.register_type(r_type);

                        module.instructions.push(Instruction::MatrixTimesScalar {
                            result_type: TypeId(mat_type),
                            result_id: ResultId(result_id),
                            matrix: ValueId(r_value),
                            scalar: ValueId(l_value),
                        });
                    },
                    (Vec(v_len), Mat(m_len)) if v_len == m_len => {
                        let vec_type = module.register_type(l_type);

                        module.instructions.push(Instruction::VectorTimesMatrix {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            vector: ValueId(l_value),
                            matrix: ValueId(r_value),
                        });
                    },
                    (Mat(m_len), Vec(v_len)) if v_len == m_len => {
                        let vec_type = module.register_type(l_type);

                        module.instructions.push(Instruction::MatrixTimesVector {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            matrix: ValueId(l_value),
                            vector: ValueId(r_value),
                        });
                    },
                    _ => return Err("Unsupported multiplication")
                }

                result_id
            },

            Node::Divide => impl_math_op!(
                args, module, "Divide",
                SDiv, FDiv
            ),

            Node::Modulus => impl_math_op!(
                args, module, "Modulus",
                SMod, FMod
            ),

            Node::Clamp => impl_glsl_call!(
                args, module,
                FClamp,
                3, TypeName::Float
            ),

            Node::Cross => impl_glsl_call!(
                args, module,
                Cross,
                2, TypeName::Float
            ),

            Node::Floor => impl_glsl_call!(
                args, module,
                Floor,
                1, TypeName::Float
            ),

            Node::Ceil => impl_glsl_call!(
                args, module,
                Ceil,
                1, TypeName::Float
            ),

            Node::Round => impl_glsl_call!(
                args, module,
                Round,
                1, TypeName::Float
            ),

            Node::Sin => impl_glsl_call!(
                args, module,
                Sin,
                1, TypeName::Float
            ),

            Node::Cos => impl_glsl_call!(
                args, module,
                Cos,
                1, TypeName::Float
            ),

            Node::Tan => impl_glsl_call!(
                args, module,
                Tan,
                1, TypeName::Float
            ),

            Node::Pow => impl_glsl_call!(
                args, module,
                Pow,
                2, TypeName::Float
            ),

            Node::Min => {
                if args.len() != 2 {
                    return Err("Wrong number of arguments for Min");
                }

                let ext_id = module.import_set(String::from("GLSL.std.450"));

                let result_id = module.get_id();

                let (l_type, _) = args[0];
                let (r_type, _) = args[1];

                let args: ::std::vec::Vec<_> = args.into_iter()
                    .map(|(_, rid)| Id(rid))
                    .collect();

                match (l_type, r_type) {
                    (Int, Int) => {
                        let int_type = module.register_type(l_type);

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(int_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: SMin as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    (Float, Float) => {
                        let float_type = module.register_type(l_type);

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(float_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: FMin as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    _ => return Err("Unsupported Min operation")
                }

                result_id
            },

            Node::Max => {
                if args.len() != 2 {
                    return Err("Wrong number of arguments for Max");
                }

                let ext_id = module.import_set(String::from("GLSL.std.450"));

                let result_id = module.get_id();

                let (l_type, _) = args[0];
                let (r_type, _) = args[1];

                let args: ::std::vec::Vec<_> = args.into_iter()
                    .map(|(_, rid)| Id(rid))
                    .collect();

                match (l_type, r_type) {
                    (Int, Int) => {
                        let int_type = module.register_type(l_type);

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(int_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: SMax as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    (Float, Float) => {
                        let float_type = module.register_type(l_type);

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(float_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: FMax as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    _ => return Err("Unsupported Max operation")
                }

                result_id
            },

            Node::Length => {
                if args.len() != 1 {
                    return Err("Wrong number of arguments for Length");
                }

                let (arg_type, arg_val) = args[0];
                if let Vec(_) = arg_type {
                    let ext_id = module.import_set(String::from("GLSL.std.450"));

                    let result_id = module.get_id();
                    let res_type = module.register_type(arg_type);

                    let args = vec![
                        Id(arg_val)
                    ];

                    module.instructions.push(Instruction::ExtInst {
                        result_type: TypeId(res_type),
                        result_id: ResultId(result_id),
                        set: ValueId(ext_id),
                        instruction: Length as u32,
                        operands: args.into_boxed_slice(),
                    });

                    result_id
                } else {
                    return Err("Unsupported Length operation");
                }
            },

            Node::Distance => {
                if args.len() != 2 {
                    return Err("Wrong number of arguments for Distance");
                }

                let ext_id = module.import_set(String::from("GLSL.std.450"));

                let result_id = module.get_id();

                let (l_type, _) = args[0];
                let (r_type, _) = args[1];

                let args: ::std::vec::Vec<_> = args.into_iter()
                    .map(|(_, rid)| Id(rid))
                    .collect();

                match (l_type, r_type) {
                    (Vec(l_size), Vec(r_size)) if l_size == r_size => {
                        let vec_type = module.register_type(l_type);

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: Distance as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    _ => return Err("Unsupported Distance operation")
                }

                result_id
            },

            Node::Reflect => {
                if args.len() != 2 {
                    return Err("Wrong number of arguments for Reflect");
                }

                let ext_id = module.import_set(String::from("GLSL.std.450"));

                let result_id = module.get_id();

                let (l_type, _) = args[0];
                let (r_type, _) = args[1];

                let args: ::std::vec::Vec<_> = args.into_iter()
                    .map(|(_, rid)| Id(rid))
                    .collect();

                match (l_type, r_type) {
                    (Vec(l_size), Vec(r_size)) if l_size == r_size => {
                        let vec_type = module.register_type(l_type);

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: Reflect as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    _ => return Err("Unsupported Reflect operation")
                }

                result_id
            },

            Node::Refract => {
                if args.len() != 3 {
                    return Err("Wrong number of arguments for Reflect");
                }

                let result_id = module.get_id();

                let (l_type, _) = args[0];
                let (r_type, _) = args[1];
                let (i_type, _) = args[2];

                match (l_type, r_type, i_type) {
                    (Vec(l_size), Vec(r_size), Float) if l_size == r_size => {
                        let vec_type = module.register_type(l_type);

                        let ext_id = module.import_set(String::from("GLSL.std.450"));

                        let args: ::std::vec::Vec<_> = args.into_iter()
                            .map(|(_, rid)| Id(rid))
                            .collect();

                        module.instructions.push(Instruction::ExtInst {
                            result_type: TypeId(vec_type),
                            result_id: ResultId(result_id),
                            set: ValueId(ext_id),
                            instruction: Refract as u32,
                            operands: args.into_boxed_slice(),
                        });
                    },
                    _ => return Err("Unsupported Refract operation")
                }

                result_id
            },

            Node::Dot => {
                if args.len() != 2 {
                    return Err("Wrong number of arguments for Dot");
                }

                let float_type = module.register_type(TypeName::Float);

                let result_id = module.get_id();

                module.instructions.push(Instruction::Dot {
                    result_type: TypeId(float_type),
                    result_id: ResultId(result_id),
                    lhs: ValueId(args[0].1),
                    rhs: ValueId(args[1].1),
                });

                result_id
            },
        })
    }
}

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
            .map(|(node, _)| Ok((node, try!(self.infer_type(node)))))
            .collect()
    }

    fn infer_type(&self, index: NodeIndex<u32>) -> Result<TypeName, &'static str> {
        let args: Vec<_> = try!(
            self.graph.neighbors_directed(index, Incoming)
                .map(|index| self.infer_type(index))
                .collect()
        );

        Ok(match self.graph[index] {
            Node::Input(_, type_name) => type_name,
            Node::Output(_, type_name) => type_name,
            Node::Constant(value) => value.to_type_name(),
            Node::Normalize => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Normalize")
                }

                args[0]
            },
            Node::Add => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Add")
                }

                args[0]
            },
            Node::Substract => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Substract")
                }

                args[0]
            },
            Node::Multiply => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Multiply")
                }

                args[0]
            },
            Node::Divide => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Divide")
                }

                args[0]
            },
            Node::Modulus => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Modulus")
                }

                args[0]
            },
            Node::Clamp => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Clamp")
                }

                args[0]
            },
            Node::Cross => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Cross")
                }

                args[0]
            },
            Node::Floor => TypeName::Int,
            Node::Ceil => TypeName::Int,
            Node::Round => TypeName::Int,
            Node::Sin => TypeName::Float,
            Node::Cos => TypeName::Float,
            Node::Tan => TypeName::Float,
            Node::Pow => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Pow")
                }

                args[0]
            },
            Node::Min => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Min")
                }

                args[0]
            },
            Node::Max => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Max")
                }

                args[0]
            },
            Node::Length => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Length")
                }

                args[0]
            },
            Node::Distance => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Distance")
                }

                args[0]
            },
            Node::Reflect => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Reflect")
                }

                args[0]
            },
            Node::Refract => {
                if args.is_empty() {
                    return Err("Not enough arguments to infer type for Refract")
                }

                args[0]
            },
            Node::Dot => TypeName::Float,
        })
    }
}
