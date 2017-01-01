use std::mem;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId
};
use spirv_utils::instruction::*;
use spirv_utils::write::to_bytecode;
pub use spirv_utils::desc::ExecutionModel as ShaderType;

use graph::*;

/// Builder struct used to define a SPIR-V module
#[derive(Debug)]
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

include!(concat!(env!("OUT_DIR"), "/module.rs"));

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
    pub fn build(graph: &Graph, mod_type: ShaderType) -> Result<Module, &'static str> {
        let mut program = Module::new(mod_type);
        for node in graph.outputs() {
            program.visit(graph, node)?;
        }

        Ok(program)
    }

    /// Acquire a new identifier to be used in the module
    pub fn get_id(&mut self) -> u32 {
        self.counter.fetch_add(1, Ordering::SeqCst) as u32
    }

    /// Get the type of this shader module
    pub fn get_type(&self) -> ShaderType {
        self.mod_type
    }

    /// Get the ID bound of this module
    pub fn bound(&self) -> u32 {
        self.counter.load(Ordering::SeqCst) as u32
    }

    /// Get the list of extensions imported by this module
    pub fn get_imports(&self) -> Vec<String> {
        self.imports.keys().cloned().collect()
    }

    /// Get the annotations intructions for this module
    pub fn get_annotations(&self) -> Vec<Instruction> {
        self.annotations.clone()
    }

    /// Get the declarations intructions for this module
    pub fn get_declarations(&self) -> Vec<Instruction> {
        self.declarations.clone()
    }

    /// Get the code intructions for this module
    pub fn get_instructions(&self) -> Vec<Instruction> {
        self.instructions.clone()
    }

    /// Get the list of inputs and outputs of this module
    pub fn get_io(&self) -> Vec<u32> {
        self.inputs.iter()
            .chain(self.outputs.iter())
            .map(|id| id.0)
            .collect()
    }

    /// Import an instruction set to the module, returning its ID
    pub fn import_set(&mut self, name: String) -> u32 {
        if let Some(ext) = self.imports.get(&name) {
            return (ext.0).0;
        }

        let ext_id = self.get_id();
        self.imports.insert(name.clone(), (ValueId(ext_id), Instruction::ExtInstImport {
            result_id: ResultId(ext_id),
            name: name.clone(),
        }));

        ext_id
    }

    /// Get the ID corresponding to a Type
    pub fn register_type(&mut self, type_id: TypeName) -> u32 {
        if let Some(reg_id) = self.types.get(&type_id) {
            return reg_id.0;
        }

        let res_id = match type_id {
            TypeName::Bool => {
                let bool_id = self.get_id();

                self.declarations.push(Instruction::TypeBool {
                    result_type: TypeId(bool_id)
                });

                bool_id
            },
            TypeName::Int(is_signed) => {
                let int_id = self.get_id();

                self.declarations.push(Instruction::TypeInt {
                    result_type: TypeId(int_id),
                    width: 32,
                    signed: is_signed
                });

                int_id
            },
            TypeName::Float(is_double) => {
                let float_id = self.get_id();

                self.declarations.push(Instruction::TypeFloat {
                    result_type: TypeId(float_id),
                    width: if is_double {
                        64
                    } else {
                        32
                    },
                });

                float_id
            },
            TypeName::Vec(len, scalar) => {
                let float_id = self.register_type(*scalar);
                let vec_id = self.get_id();

                self.declarations.push(Instruction::TypeVector {
                    result_type: TypeId(vec_id),
                    type_id: TypeId(float_id),
                    len: len,
                });

                vec_id
            },
            TypeName::Mat(len, scalar) => {
                let float_id = self.register_type(*scalar);
                let mat_id = self.get_id();

                self.declarations.push(Instruction::TypeMatrix {
                    result_type: TypeId(mat_id),
                    type_id: TypeId(float_id),
                    cols: len,
                });

                mat_id
            },
        };

        self.types.insert(type_id, TypeId(res_id));
        res_id
    }

    /// Add a new constant to the module, returning its ID
    pub fn register_constant(&mut self, constant: TypedValue) -> Result<u32, &'static str> {
        impl_register_constant!(self, constant)
    }

    fn visit(&mut self, graph: &Graph, node: NodeIndex<u32>) -> Result<u32, &'static str> {
        let args = graph.arguments(node)?;

        let args: Vec<_> = try!(
            args.iter().map(|&(_, tname)| Ok(tname))
                .zip(
                    args.iter().map(|edge| self.visit(graph, edge.0))
                )
                .map(|(a, b)| Ok((a?, b?)))
                .collect()
        );

        graph.node(node).get_result(self, args)
    }

    /// Build the module, returning a list of instructions
    pub fn get_operations(&self) -> Vec<Instruction> {
        let mut result = Vec::with_capacity(
            self.imports.len() +
            self.annotations.len() +
            self.declarations.len() +
            self.instructions.len() + 8
        );

        // Capabilities
        result.push(Instruction::Capability {
            capability: desc::Capability::Shader
        });

        // Extensions import
        for &(_, ref op) in self.imports.values() {
            result.push(op.clone());
        }

        // Memory Model
        result.push(Instruction::MemoryModel {
            addressing_model: desc::AddressingModel::Logical,
            memory_model: desc::MemoryModel::GLSL450
        });

        // Entry points
        let prog_io: Vec<Id> = self.get_io()
            .into_iter()
            .map(|i| Id(i))
            .collect();
        result.push(Instruction::EntryPoint {
            execution_model: self.mod_type,
            func: ENTRY_ID.to_value_id(),
            name: String::from("main"),
            interface: prog_io.into_boxed_slice()
        });

        // Execution Models
        result.push(Instruction::ExecutionMode {
            entry: ENTRY_ID.to_value_id(),
            execution_mode: ExecutionMode::OriginUpperLeft
        });

        // Annotations
        result.append(&mut self.annotations.clone());

        // Declarations
        result.append(&mut self.declarations.clone());

        // Functions
        result.push(Instruction::Function {
            result_type: VOID_ID,
            result_id: ENTRY_ID,
            function_control: desc::FunctionControl::empty(),
            fn_ty: FUNC_ID
        });
        result.push(Instruction::Label {
            result_id: LABEL_ID
        });

        result.append(&mut self.instructions.clone());

        result.push(Instruction::Return);
        result.push(Instruction::FunctionEnd);

        result
    }

    /// Get the instructions of the module in binary form
    pub fn get_bytecode(&self) -> Vec<u8> {
        let operations = self.get_operations();

        let mut res = Vec::with_capacity(operations.len() + 5);

        res.push(0x07230203u32);
        res.push(0x00010000);
        res.push(0x000c0001);
        res.push(self.bound());
        res.push(0);

        for op in operations {
            res.append(&mut to_bytecode(op));
        }

        res.into_iter()
            .flat_map(|words| unsafe {
                let as_bytes = mem::transmute::<u32, [u8; 4]>(words);
                vec![as_bytes[0], as_bytes[1], as_bytes[2], as_bytes[3]]
            })
            .collect()
    }
}
