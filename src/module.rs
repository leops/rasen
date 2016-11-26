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
            try!(program.visit(graph, node));
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
            TypeName::Int => {
                let int_id = self.get_id();

                self.declarations.push(Instruction::TypeInt {
                    result_type: TypeId(int_id),
                    width: 32,
                    signed: true
                });

                int_id
            },
            TypeName::Float => {
                let float_id = self.get_id();

                self.declarations.push(Instruction::TypeFloat {
                    result_type: TypeId(float_id),
                    width: 32,
                });

                float_id
            },
            TypeName::Vec(len) => {
                let float_id = self.register_type(TypeName::Float);
                let vec_id = self.get_id();

                self.declarations.push(Instruction::TypeVector {
                    result_type: TypeId(vec_id),
                    type_id: TypeId(float_id),
                    len: len,
                });

                vec_id
            },
            TypeName::Mat(len) => {
                let float_id = self.register_type(TypeName::Float);
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
        Ok(match constant {
            TypedValue::Vec4(x, y, z, w) => {
                let x_id = ValueId(try!(self.register_constant(TypedValue::Float(x))));
                let y_id = ValueId(try!(self.register_constant(TypedValue::Float(y))));
                let z_id = ValueId(try!(self.register_constant(TypedValue::Float(z))));
                let w_id = ValueId(try!(self.register_constant(TypedValue::Float(w))));

                let vec_type = self.register_type(TypeName::Vec(4));
                let res_id = self.get_id();
                self.declarations.push(Instruction::ConstantComposite {
                    result_type: TypeId(vec_type),
                    result_id: ResultId(res_id),
                    flds: vec![
                        x_id, y_id, z_id, w_id
                    ].into_boxed_slice(),
                });

                res_id
            },
            TypedValue::Vec3(x, y, z) => {
                let x_id = ValueId(try!(self.register_constant(TypedValue::Float(x))));
                let y_id = ValueId(try!(self.register_constant(TypedValue::Float(y))));
                let z_id = ValueId(try!(self.register_constant(TypedValue::Float(z))));

                let vec_type = self.register_type(TypeName::Vec(3));
                let res_id = self.get_id();
                self.declarations.push(Instruction::ConstantComposite {
                    result_type: TypeId(vec_type),
                    result_id: ResultId(res_id),
                    flds: vec![x_id, y_id, z_id].into_boxed_slice(),
                });

                res_id
            },
            TypedValue::Vec2(x, y) => {
                let x_id = ValueId(try!(self.register_constant(TypedValue::Float(x))));
                let y_id = ValueId(try!(self.register_constant(TypedValue::Float(y))));

                let vec_type = self.register_type(TypeName::Vec(2));
                let res_id = self.get_id();
                self.declarations.push(Instruction::ConstantComposite {
                    result_type: TypeId(vec_type),
                    result_id: ResultId(res_id),
                    flds: vec![x_id, y_id].into_boxed_slice(),
                });

                res_id
            },
            TypedValue::Float(value) => {
                let float_type = self.register_type(TypeName::Float);
                let res_id = self.get_id();

                unsafe {
                    let transmuted = mem::transmute::<f32, u32>(value);
                    self.declarations.push(Instruction::Constant {
                        result_type: TypeId(float_type),
                        result_id: ResultId(res_id),
                        val: vec![transmuted].into_boxed_slice(),
                    });
                }

                res_id
            },
            TypedValue::Int(value) => {
                let int_type = self.register_type(TypeName::Int);
                let res_id = self.get_id();

                unsafe {
                    let transmuted = mem::transmute::<i32, u32>(value);
                    self.declarations.push(Instruction::Constant {
                        result_type: TypeId(int_type),
                        result_id: ResultId(res_id),
                        val: vec![transmuted].into_boxed_slice(),
                    });
                }

                res_id
            },
            TypedValue::Bool(value) => {
                let bool_type = self.register_type(TypeName::Bool);
                let res_id = self.get_id();

                if value {
                    self.declarations.push(Instruction::ConstantTrue {
                        result_type: TypeId(bool_type),
                        result_id: ResultId(res_id),
                    });
                } else {
                    self.declarations.push(Instruction::ConstantFalse {
                        result_type: TypeId(bool_type),
                        result_id: ResultId(res_id),
                    });
                }

                res_id
            },
            _ => return Err("Unimplemented constant type"),
        })
    }

    fn visit(&mut self, graph: &Graph, node: NodeIndex<u32>) -> Result<u32, &'static str> {
        let args = try!(graph.arguments(node));

        let first: Vec<_> = args.iter().map(|&(_, tname)| tname).collect();
        let second: Vec<_> = try!(args.iter().map(|edge| self.visit(graph, edge.0)).collect());

        let args: Vec<_> = first.into_iter()
            .zip(second.into_iter())
            .collect();

        graph.node(node).get_result(self, args)
    }

    /// Build the module, returning a list of instructions
    pub fn get_operations(&self) -> Vec<Instruction> {
        let prog_io: Vec<Id> = self.get_io()
            .into_iter()
            .map(|i| Id(i))
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
            self.bound(),
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
}
