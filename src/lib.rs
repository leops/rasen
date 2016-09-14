//! Build a SPIR-V module from an operation graph
//!
//! This library lets you define a shader module as a graph (using the `petgraph` library) of
//! `Node`, describing the operations needed to obtain the outputs of the shader.
//!
//! ```
//! extern crate petgraph;
//! extern crate rasen;
//!
//! use petgraph::Graph;
//! use rasen::*;
//!
//! fn main() {
//!   let mut graph = Graph::<Node, ()>::new();
//!
//!   // A vec3 input at location 0
//!   let normal = graph.add_node(Node::Input(0, TypeName::Vec3));
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
//!   let color = graph.add_node(Node::Output(0, TypeName::Vec4));
//!
//!   // Normalize the normal
//!   graph.add_edge(normal, normalize, ());
//!
//!   // Compute the dot product of the surface normal and the light direction
//!   graph.add_edge(normalize, dot, ());
//!   graph.add_edge(light_dir, dot, ());
//!
//!   // Restrict the result into the ambient light range
//!   graph.add_edge(dot, clamp, ());
//!   graph.add_edge(min_light, clamp, ());
//!   graph.add_edge(max_light, clamp, ());
//!
//!   // Multiply the light intensity by the surface color
//!   graph.add_edge(clamp, multiply, ());
//!   graph.add_edge(mat_color, multiply, ());
//!
//!   // Write the result to the output
//!   graph.add_edge(multiply, color, ());
//!
//!   let bytecode = build_program(&graph);
//!   // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
//! }
//! ```
//!
//! On a lower level, you can use the `Module` struct to build your module by adding operations
//! directly into it.
//!
//! Operations can be transformed to bytecode using the `bytecode` module, and specifically the
//! `to_bytecode` function.
//!

extern crate petgraph;

pub mod spirv;
pub mod bytecode;
pub mod graph;

use std::mem;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use petgraph::graph::NodeIndex;
use petgraph::{
    Graph,
    Outgoing, Incoming
};

use spirv::*;
pub use graph::*;
pub use bytecode::to_bytecode;

/// Builder struct used to define a SPIR-V module
#[derive(Default)]
pub struct Module {
    pub inputs: Vec<u32>,
    pub outputs: Vec<u32>,
    imports: HashMap<String, (u32, Operation)>,
    pub annotations: Vec<Operation>,
    types: HashMap<TypeName, u32>,
    pub declarations: Vec<Operation>,
    pub instructions: Vec<Operation>,
    counter: AtomicUsize,
}

const VOID_ID: u32 = 1;
const FUNC_ID: u32 = 2;
const LABEL_ID: u32 = 3;
const ENTRY_ID: u32 = 4;

impl Module {
    /// Create a new shader module with some predefined base values
    pub fn new() -> Module {
        Module {
            declarations: vec![
                Operation::OpTypeVoid(VOID_ID),
                Operation::OpTypeFunction(FUNC_ID, VOID_ID)
            ],
            counter: AtomicUsize::new(5),
            .. Default::default()
        }
    }

    /// Create a new Module and add instructions to it based on a Graph
    pub fn build(graph: &Graph<Node, ()>) -> Module {
        let mut program = Module::new();
        for node in graph.externals(Outgoing) {
            program.visit(graph, node);
        }

        program
    }

    /// Acquire a new identifier to be used in the module
    pub fn get_id(&mut self) -> u32 {
        self.counter.fetch_add(1, Ordering::SeqCst) as u32
    }

    /// Import an instruction set to the module, returning its ID
    pub fn import_set(&mut self, name: String) -> u32 {
        if let Some(ext) = self.imports.get(&name) {
            return ext.0;
        }

        let ext_id = self.get_id();
        self.imports.insert(name.clone(), (ext_id, Operation::OpExtInstImport(ext_id, name.clone())));
        ext_id
    }

    /// Get the ID corresponding to a Type
    pub fn register_type(&mut self, type_id: TypeName) -> u32 {
        if let Some(reg_id) = self.types.get(&type_id) {
            return *reg_id;
        }

        match type_id {
            TypeName::Vec4 => {
                let float_id = self.register_type(TypeName::Float);
                let vec4_id = self.get_id();

                self.declarations.push(Operation::OpTypeVector(vec4_id, float_id, 4));
                self.types.insert(TypeName::Vec4, vec4_id);

                vec4_id
            },
            TypeName::Vec3 => {
                let float_id = self.register_type(TypeName::Float);
                let vec3_id = self.get_id();

                self.declarations.push(Operation::OpTypeVector(vec3_id, float_id, 3));
                self.types.insert(TypeName::Vec3, vec3_id);

                vec3_id
            },
            TypeName::Vec2 => {
                let float_id = self.register_type(TypeName::Float);
                let vec2_id = self.get_id();

                self.declarations.push(Operation::OpTypeVector(vec2_id, float_id, 2));
                self.types.insert(TypeName::Vec2, vec2_id);

                vec2_id
            },
            TypeName::Float => {
                let float_id = self.get_id();

                self.declarations.push(Operation::OpTypeFloat(float_id, 32));
                self.types.insert(TypeName::Float, float_id);

                float_id
            },
        }
    }

    /// Add a new constant to the module, returning its ID
    pub fn register_constant(&mut self, constant: TypedValue) -> u32 {
        match constant {
            TypedValue::Vec4(x, y, z, w) => {
                let x_id = self.register_constant(TypedValue::Float(x));
                let y_id = self.register_constant(TypedValue::Float(y));
                let z_id = self.register_constant(TypedValue::Float(z));
                let w_id = self.register_constant(TypedValue::Float(w));

                let vec_type = self.register_type(TypeName::Vec4);
                let res_id = self.get_id();
                self.declarations.push(Operation::OpConstantComposite(res_id, vec_type, vec![x_id, y_id, z_id, w_id]));

                res_id
            },
            TypedValue::Vec3(x, y, z) => {
                let x_id = self.register_constant(TypedValue::Float(x));
                let y_id = self.register_constant(TypedValue::Float(y));
                let z_id = self.register_constant(TypedValue::Float(z));

                let vec_type = self.register_type(TypeName::Vec3);
                let res_id = self.get_id();
                self.declarations.push(Operation::OpConstantComposite(res_id, vec_type, vec![x_id, y_id, z_id]));

                res_id
            },
            TypedValue::Vec2(x, y) => {
                let x_id = self.register_constant(TypedValue::Float(x));
                let y_id = self.register_constant(TypedValue::Float(y));

                let vec_type = self.register_type(TypeName::Vec2);
                let res_id = self.get_id();
                self.declarations.push(Operation::OpConstantComposite(res_id, vec_type, vec![x_id, y_id]));

                res_id
            },
            TypedValue::Float(value) => {
                let float_type = self.register_type(TypeName::Float);
                let res_id = self.get_id();

                unsafe {
                    let transmuted = mem::transmute::<f32, u32>(value);
                    self.declarations.push(Operation::OpConstant(res_id, float_type, transmuted));
                }

                res_id
            }
        }
    }

    /// Build the module, returning a list of operations
    pub fn get_operations(&self) -> Vec<Operation> {
        let prog_io: Vec<u32> = self.inputs.iter()
            .chain(self.outputs.iter())
            .cloned()
            .collect();

        vec![
            // Capabilities
            Operation::OpCapability(Capability::Shader),
        ].iter().chain(
            // Extensions import
            self.imports.values()
                .map(|&(_, ref op)| op)
        ).chain(vec![
            // Memory Model
            Operation::OpMemoryModel(AddressingModel::Logical, MemoryModel::GLSL450),

            // Entry points
            Operation::OpEntryPoint(ExecutionModel::Fragment, ENTRY_ID, String::from("main"), prog_io),

            // Execution Models
            Operation::OpExecutionMode(ENTRY_ID, ExecutionMode::OriginUpperLeft),
        ].iter()).chain(
            // Annotations
            self.annotations.iter()
        ).chain(
            // Declarations
            self.declarations.iter()
        ).chain(vec![
            // Functions
            Operation::OpFunction(ENTRY_ID, VOID_ID, Default::default(), FUNC_ID),
            Operation::OpLabel(LABEL_ID),
        ].iter()).chain(
            self.instructions.iter()
        ).chain(vec![
            Operation::OpReturn,
            Operation::OpFunctionEnd,
        ].iter()).map(|r| r.clone()).collect()
    }

    fn visit(&mut self, graph: &Graph<Node, ()>, node: NodeIndex<u32>) -> u32 {
        match *graph.node_weight(node).unwrap() {
            Node::Output(location, ref attr_type) => {
                let type_id = self.register_type(*attr_type);

                let ptr_type = self.get_id();
                self.declarations.push(Operation::OpTypePointer(ptr_type, StorageClass::Output, type_id));

                let var_id = self.get_id();
                self.outputs.push(var_id);
                self.declarations.push(Operation::OpVariable(var_id, ptr_type, StorageClass::Output));

                self.annotations.push(Operation::OpDecorate(var_id, Decoration::Location, location));

                let incoming_node = graph.neighbors_directed(node, Incoming).next();
                let value_id = self.visit(graph, incoming_node.unwrap());
                self.instructions.push(Operation::OpStore(var_id, value_id));

                var_id
            },
            Node::Input(location, ref attr_type) => {
                let type_id = self.register_type(*attr_type);

                let ptr_type = self.get_id();
                self.declarations.push(Operation::OpTypePointer(ptr_type, StorageClass::Input, type_id));

                let var_id = self.get_id();
                self.inputs.push(var_id);
                self.declarations.push(Operation::OpVariable(var_id, ptr_type, StorageClass::Input));

                self.annotations.push(Operation::OpDecorate(var_id, Decoration::Location, location));

                let res_id = self.get_id();
                self.instructions.push(Operation::OpLoad(res_id, type_id, var_id));

                res_id
            },
            Node::Constant(ref const_type) => {
                self.register_constant(*const_type)
            },
            Node::Normalize => {
                let ext_id = self.import_set(String::from("GLSL.std.450"));

                let float_type = self.register_type(TypeName::Vec3);

                let args: Vec<u32> = graph.neighbors_directed(node, Incoming)
                    .map(|index| self.visit(graph, index))
                    .collect();

                let result_id = self.get_id();
                self.instructions.push(Operation::OpExtInst(result_id, float_type, ext_id, 69, args));

                result_id
            },
            Node::Multiply => {
                let vec_type = self.register_type(TypeName::Vec4);

                let args: Vec<u32> = graph.neighbors_directed(node, Incoming)
                    .map(|index| self.visit(graph, index))
                    .collect();

                let result_id = self.get_id();
                self.instructions.push(Operation::OpVectorTimesScalar(result_id, vec_type, args[0], args[1]));

                result_id
            },
            Node::Clamp => {
                let ext_id = self.import_set(String::from("GLSL.std.450"));

                let float_type = self.register_type(TypeName::Float);

                let args: Vec<u32> = graph.neighbors_directed(node, Incoming)
                    .map(|index| self.visit(graph, index))
                    .collect();

                let result_id = self.get_id();
                self.instructions.push(Operation::OpExtInst(result_id, float_type, ext_id, 43, args));

                result_id
            },
            Node::Dot => {
                let float_type = self.register_type(TypeName::Float);

                let args: Vec<u32> = graph.neighbors_directed(node, Incoming)
                    .map(|index| self.visit(graph, index))
                    .collect();

                let result_id = self.get_id();
                self.instructions.push(Operation::OpDot(result_id, float_type, args[0], args[1]));

                result_id
            },
        }
    }
}

/// Transform a node graph to SPIR-V bytecode
pub fn build_program(graph: &Graph<Node, ()>) -> Vec<u8> {
    let program = Module::build(graph);
    let operations = program.get_operations();
    to_bytecode(operations)
}
