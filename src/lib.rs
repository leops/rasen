//! Build a SPIR-V module from an operation graph
//!
//! This library lets you define a shader module as a graph (using the `petgraph` library) of
//! `Node`, describing the operations needed to obtain the outputs of the shader.
//!
//! ```
//! extern crate rasen;
//!
//! use rasen::*;
//!
//! fn main() {
//!   let mut graph = Graph::new();
//!
//!   // A vec3 input at location 0
//!   let normal = graph.add_node(Node::Input(0, TypeName::Vec(3)));
//!
//!   // Some ambient light constants
//!   let min_light = graph.add_node(Node::Constant(TypedValue::Float(0.1)));
//!   let max_light = graph.add_node(Node::Constant(TypedValue::Float(1.0)));
//!   let light_dir = graph.add_node(Node::Constant(TypedValue::Vec3(0.3, -0.5, 0.2)));
//!
//!   // The Material color (also a constant)
//!   let mat_color = graph.add_node(Node::Constant(TypedValue::Vec4(0.25, 0.625, 1.0, 1.0)));
//!
//!   // Some usual function calls
//!   let normalize = graph.add_node(Node::Normalize);
//!   let dot = graph.add_node(Node::Dot);
//!   let clamp = graph.add_node(Node::Clamp);
//!   let multiply = graph.add_node(Node::Multiply);
//!
//!   // And a vec4 output at location 0
//!   let color = graph.add_node(Node::Output(0, TypeName::Vec(4)));
//!
//!   // Normalize the normal
//!   graph.add_edge(normal, normalize);
//!
//!   // Compute the dot product of the surface normal and the light direction
//!   graph.add_edge(normalize, dot);
//!   graph.add_edge(light_dir, dot);
//!
//!   // Restrict the result into the ambient light range
//!   graph.add_edge(dot, clamp);
//!   graph.add_edge(min_light, clamp);
//!   graph.add_edge(max_light, clamp);
//!
//!   // Multiply the light intensity by the surface color
//!   graph.add_edge(clamp, multiply);
//!   graph.add_edge(mat_color, multiply);
//!
//!   // Write the result to the output
//!   graph.add_edge(multiply, color);
//!
//!   let bytecode = build_program(&graph, ShaderType::Fragment);
//!   // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
//! }
//! ```
//!
//! On a lower level, you can use the `Module` struct to build your module by adding instructions
//! directly into it.
//!

extern crate petgraph;
extern crate spirv_utils;

pub mod graph;
pub mod glsl;

use std::mem;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId
};
use spirv_utils::instruction::*;
use spirv_utils::write::to_bytecode;

/// Re-export of spirv_utils::desc::ExecutionModel used to define the type of a shader module
pub use spirv_utils::desc::ExecutionModel as ShaderType;

pub use graph::*;

/// Builder struct used to define a SPIR-V module
pub struct Module {
    mod_type: ShaderType,
    pub inputs: Vec<Id>,
    pub outputs: Vec<Id>,
    imports: HashMap<String, (ValueId, Instruction)>,
    pub annotations: Vec<Instruction>,
    types: HashMap<TypeName, TypeId>,
    pub declarations: Vec<Instruction>,
    pub instructions: Vec<Instruction>,
    counter: AtomicUsize,
}

const VOID_ID: TypeId = TypeId(1);
const FUNC_ID: TypeId = TypeId(2);
const LABEL_ID: ResultId = ResultId(3);
const ENTRY_ID: ResultId = ResultId(4);

impl Module {
    /// Create a new shader module with some predefined base values
    pub fn new(mod_type: ShaderType) -> Module {
        Module {
            mod_type: mod_type,
            declarations: vec![
                Instruction::TypeVoid {
                    result_type: VOID_ID,
                },
                Instruction::TypeFunction {
                    result_type: FUNC_ID,
                    return_ty: VOID_ID,
                    params: Default::default(),
                }
            ],
            counter: AtomicUsize::new(5),

            inputs: Default::default(),
            outputs: Default::default(),
            imports: Default::default(),
            annotations: Default::default(),
            types: Default::default(),
            instructions: Default::default(),
        }
    }

    /// Create a new Module and add instructions to it based on a Graph
    pub fn build(graph: &Graph, mod_type: ShaderType) -> Module {
        let mut program = Module::new(mod_type);
        for node in graph.outputs() {
            program.visit(graph, node);
        }

        program
    }

    /// Acquire a new identifier to be used in the module
    pub fn get_id(&mut self) -> Id {
        Id(self.counter.fetch_add(1, Ordering::SeqCst) as u32)
    }

    /// Import an instruction set to the module, returning its ID
    pub fn import_set(&mut self, name: String) -> ValueId {
        if let Some(ext) = self.imports.get(&name) {
            return ext.0;
        }

        let ext_id = self.get_id();
        self.imports.insert(name.clone(), (ext_id.to_value_id(), Instruction::ExtInstImport {
            result_id: ext_id.to_result_id(),
            name: name.clone(),
        }));

        ext_id.to_value_id()
    }

    /// Get the ID corresponding to a Type
    pub fn register_type(&mut self, type_id: TypeName) -> TypeId {
        if let Some(reg_id) = self.types.get(&type_id) {
            return *reg_id;
        }

        let res_id = match type_id {
            TypeName::Bool => {
                let bool_id = self.get_id().to_type_id();

                self.declarations.push(Instruction::TypeBool {
                    result_type: bool_id
                });

                bool_id
            },
            TypeName::Int => {
                let int_id = self.get_id().to_type_id();

                self.declarations.push(Instruction::TypeInt {
                    result_type: int_id,
                    width: 32,
                    signed: true
                });

                int_id
            },
            TypeName::Float => {
                let float_id = self.get_id().to_type_id();

                self.declarations.push(Instruction::TypeFloat {
                    result_type: float_id,
                    width: 32,
                });

                float_id
            },
            TypeName::Vec(len) => {
                let float_id = self.register_type(TypeName::Float);
                let vec_id = self.get_id().to_type_id();

                self.declarations.push(Instruction::TypeVector {
                    result_type: vec_id,
                    type_id: float_id,
                    len: len,
                });

                vec_id
            },
            TypeName::Mat(len) => {
                let float_id = self.register_type(TypeName::Float);
                let mat_id = self.get_id().to_type_id();

                self.declarations.push(Instruction::TypeMatrix {
                    result_type: mat_id,
                    type_id: float_id,
                    cols: len,
                });

                mat_id
            },
        };

        self.types.insert(type_id, res_id);
        res_id
    }

    /// Add a new constant to the module, returning its ID
    pub fn register_constant(&mut self, constant: TypedValue) -> ResultId {
        match constant {
            TypedValue::Vec4(x, y, z, w) => {
                let x_id = self.register_constant(TypedValue::Float(x)).to_value_id();
                let y_id = self.register_constant(TypedValue::Float(y)).to_value_id();
                let z_id = self.register_constant(TypedValue::Float(z)).to_value_id();
                let w_id = self.register_constant(TypedValue::Float(w)).to_value_id();

                let vec_type = self.register_type(TypeName::Vec(4));
                let res_id = self.get_id().to_result_id();
                self.declarations.push(Instruction::ConstantComposite {
                    result_type: vec_type,
                    result_id: res_id,
                    flds: vec![x_id, y_id, z_id, w_id].into_boxed_slice(),
                });

                ResultId(res_id.0)
            },
            TypedValue::Vec3(x, y, z) => {
                let x_id = self.register_constant(TypedValue::Float(x)).to_value_id();
                let y_id = self.register_constant(TypedValue::Float(y)).to_value_id();
                let z_id = self.register_constant(TypedValue::Float(z)).to_value_id();

                let vec_type = self.register_type(TypeName::Vec(3));
                let res_id = self.get_id().to_result_id();
                self.declarations.push(Instruction::ConstantComposite {
                    result_type: vec_type,
                    result_id: res_id,
                    flds: vec![x_id, y_id, z_id].into_boxed_slice(),
                });

                ResultId(res_id.0)
            },
            TypedValue::Vec2(x, y) => {
                let x_id = self.register_constant(TypedValue::Float(x)).to_value_id();
                let y_id = self.register_constant(TypedValue::Float(y)).to_value_id();

                let vec_type = self.register_type(TypeName::Vec(2));
                let res_id = self.get_id().to_result_id();
                self.declarations.push(Instruction::ConstantComposite {
                    result_type: vec_type,
                    result_id: res_id,
                    flds: vec![x_id, y_id].into_boxed_slice(),
                });

                ResultId(res_id.0)
            },
            TypedValue::Float(value) => {
                let float_type = self.register_type(TypeName::Float);
                let res_id = self.get_id().to_result_id();

                unsafe {
                    let transmuted = mem::transmute::<f32, u32>(value);
                    self.declarations.push(Instruction::Constant {
                        result_type: float_type,
                        result_id: res_id,
                        val: vec![transmuted].into_boxed_slice(),
                    });
                }

                ResultId(res_id.0)
            },
            TypedValue::Int(value) => {
                let int_type = self.register_type(TypeName::Int);
                let res_id = self.get_id().to_result_id();

                unsafe {
                    let transmuted = mem::transmute::<i32, u32>(value);
                    self.declarations.push(Instruction::Constant {
                        result_type: int_type,
                        result_id: res_id,
                        val: vec![transmuted].into_boxed_slice(),
                    });
                }

                ResultId(res_id.0)
            },
            TypedValue::Bool(value) => {
                let bool_type = self.register_type(TypeName::Bool);
                let res_id = self.get_id().to_result_id();

                if value {
                    self.declarations.push(Instruction::ConstantTrue {
                        result_type: bool_type,
                        result_id: res_id,
                    });
                } else {
                    self.declarations.push(Instruction::ConstantFalse {
                        result_type: bool_type,
                        result_id: res_id,
                    });
                }

                ResultId(res_id.0)
            },
            _ => unimplemented!(),
        }
    }

    /// Build the module, returning a list of instructions
    pub fn get_operations(&self) -> Vec<Instruction> {
        let prog_io: Vec<Id> = self.inputs.iter()
            .chain(self.outputs.iter())
            .cloned()
            .collect();

        vec![
            // Capabilities
            Instruction::Capability {
                capability: desc::Capability::Shader
            },
        ].iter().chain(
            // Extensions import
            self.imports.values()
                .map(|&(_, ref op)| op)
        ).chain(vec![
            // Memory Model
            Instruction::MemoryModel {
                addressing_model: desc::AddressingModel::Logical,
                memory_model: desc::MemoryModel::GLSL450
            },

            // Entry points
            Instruction::EntryPoint {
                execution_model: self.mod_type,
                func: ENTRY_ID.to_value_id(),
                name: String::from("main"),
                interface: prog_io.into_boxed_slice()
            },

            // Execution Models
            Instruction::ExecutionMode {
                entry: ENTRY_ID.to_value_id(),
                execution_mode: ExecutionMode::OriginUpperLeft
            },
        ].iter()).chain(
            // Annotations
            self.annotations.iter()
        ).chain(
            // Declarations
            self.declarations.iter()
        ).chain(vec![
            // Functions
            Instruction::Function {
                result_type: VOID_ID,
                result_id: ENTRY_ID,
                function_control: desc::FunctionControl::empty(),
                fn_ty: FUNC_ID
            },
            Instruction::Label {
                result_id: LABEL_ID
            },
        ].iter()).chain(
            self.instructions.iter()
        ).chain(vec![
            Instruction::Return,
            Instruction::FunctionEnd,
        ].iter()).map(|r| r.clone()).collect()
    }

    /// Get the instructions of the module in binary form
    pub fn get_bytecode(&self) -> Vec<u8> {
        vec![
            0x07230203u32,
            0x00010000,
            0x000c0001,
            self.counter.load(Ordering::SeqCst) as u32,
            0
        ].into_iter()
            .chain(
                self.get_operations().into_iter()
                    .flat_map(|op| to_bytecode(op))
            )
            .flat_map(|words| unsafe {
                let as_bytes = mem::transmute::<u32, [u8; 4]>(words);
                vec![as_bytes[0], as_bytes[1], as_bytes[2], as_bytes[3]]
            })
            .collect()
    }

    fn visit(&mut self, graph: &Graph, node: NodeIndex<u32>) -> ResultId {
        let args: Vec<(TypeName, ResultId)> = graph.arguments(node)
            .map(|edge| (*edge.1, self.visit(graph, edge.0)))
            .collect();

        graph.node(node).get_result(self, args)
    }
}

/// Transform a node graph to SPIR-V bytecode
pub fn build_program(graph: &Graph, mod_type: ShaderType) -> Vec<u8> {
    let program = Module::build(graph, mod_type);
    program.get_bytecode()
}
