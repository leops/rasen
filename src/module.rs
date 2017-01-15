use std::mem;
use std::slice;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use petgraph::graph::NodeIndex;

use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId
};
use spirv_utils::instruction::*;
use spirv_utils::write::to_bytecode;
pub use spirv_utils::desc::ExecutionModel as ShaderType;

use graph::*;
use errors::*;

#[derive(Debug, Eq, PartialEq, Hash)]
enum CachedConstant {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(bool, i16, u32),
    Double(bool, i16, u64),
}

impl CachedConstant {
    #[inline]
    pub fn from_f64(val: f64) -> Self {
        let bits: u64 = unsafe { mem::transmute(val) };
        let sign = bits >> 63 == 0;
        let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
        let mantissa = if exponent == 0 {
            (bits & 0xfffffffffffff) << 1
        } else {
            (bits & 0xfffffffffffff) | 0x10000000000000
        };

        exponent -= 1023 + 52;
        CachedConstant::Double(sign, exponent, mantissa)
    }

    #[inline]
    fn from_f32(f: f32) -> Self {
        let bits: u32 = unsafe { mem::transmute(f) };
        let sign = bits >> 31 == 0;
        let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
        let mantissa = if exponent == 0 {
            (bits & 0x7fffff) << 1
        } else {
            (bits & 0x7fffff) | 0x800000
        };

        exponent -= 127 + 23;
        CachedConstant::Float(sign, exponent, mantissa)
    }
}

/// Builder struct used to define a SPIR-V module
#[derive(Debug)]
pub struct Module {
    mod_type: ShaderType,
    counter: AtomicUsize,
    pub inputs: Vec<Id>,
    pub outputs: Vec<Id>,
    imports: HashMap<&'static str, (ValueId, Instruction)>,
    pub annotations: Vec<Instruction>,
    pub declarations: Vec<Instruction>,
    pub instructions: Vec<Instruction>,

    types: HashMap<&'static TypeName, TypeId>,
    constants: HashMap<CachedConstant, u32>,
    results: HashMap<NodeIndex<u32>, (&'static TypeName, u32)>,
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
            counter: AtomicUsize::new(5),
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

            inputs: Default::default(),
            outputs: Default::default(),
            imports: Default::default(),
            annotations: Default::default(),
            instructions: Default::default(),
            types: Default::default(),
            constants: Default::default(),
            results: Default::default(),
        }
    }

    /// Create a new Module and add instructions to it based on a Graph
    pub fn build(graph: &Graph, mod_type: ShaderType) -> Result<Module> {
        if graph.has_cycle() {
            return Err(ErrorKind::CyclicGraph.into());
        }

        let mut program = Module::new(mod_type);
        for node in graph.outputs() {
            program.visit(graph, node)?;
        }

        Ok(program)
    }

    /// Acquire a new identifier to be used in the module
    #[inline]
    pub fn get_id(&mut self) -> u32 {
        self.counter.fetch_add(1, Ordering::SeqCst) as u32
    }

    /// Get the type of this shader module
    #[inline]
    pub fn get_type(&self) -> ShaderType {
        self.mod_type
    }

    /// Get the ID bound of this module
    #[inline]
    pub fn bound(&self) -> u32 {
        self.counter.load(Ordering::SeqCst) as u32
    }

    /// Get the list of extensions imported by this module
    #[inline]
    pub fn get_imports(&self) -> Vec<&'static str> {
        self.imports.keys().cloned().collect()
    }

    /// Get the annotations intructions for this module
    #[inline]
    pub fn get_annotations(&self) -> Vec<Instruction> {
        self.annotations.clone()
    }

    /// Get the declarations intructions for this module
    #[inline]
    pub fn get_declarations(&self) -> Vec<Instruction> {
        self.declarations.clone()
    }

    /// Get the code intructions for this module
    #[inline]
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
    pub fn import_set(&mut self, name: &'static str) -> ValueId {
        if let Some(&(id, _)) = self.imports.get(&name) {
            return id;
        }

        let ext_id = self.get_id();
        self.imports.insert(name, (ValueId(ext_id), Instruction::ExtInstImport {
            result_id: ResultId(ext_id),
            name: String::from(name),
        }));

        ValueId(ext_id)
    }

    /// Get the ID corresponding to a Type
    pub fn register_type(&mut self, type_id: &'static TypeName) -> TypeId {
        if let Some(reg_id) = self.types.get(type_id) {
            return *reg_id;
        }

        let res_id = TypeId(match type_id {
            &TypeName::Bool => {
                let bool_id = self.get_id();

                self.declarations.push(Instruction::TypeBool {
                    result_type: TypeId(bool_id)
                });

                bool_id
            },
            &TypeName::Int(is_signed) => {
                let int_id = self.get_id();

                self.declarations.push(Instruction::TypeInt {
                    result_type: TypeId(int_id),
                    width: 32,
                    signed: is_signed
                });

                int_id
            },
            &TypeName::Float(is_double) => {
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
            &TypeName::Vec(len, scalar) => {
                let float_id = self.register_type(scalar);
                let vec_id = self.get_id();

                self.declarations.push(Instruction::TypeVector {
                    result_type: TypeId(vec_id),
                    type_id: float_id,
                    len: len,
                });

                vec_id
            },
            &TypeName::Mat(len, scalar) => {
                let float_id = self.register_type(scalar);
                let mat_id = self.get_id();

                self.declarations.push(Instruction::TypeMatrix {
                    result_type: TypeId(mat_id),
                    type_id: float_id,
                    cols: len,
                });

                mat_id
            },
        });

        self.types.insert(type_id, res_id);
        res_id
    }

    /// Add a new constant to the module, returning its ID
    pub fn register_constant(&mut self, constant: &TypedValue) -> Result<u32> {
        let cache = match constant {
            &TypedValue::Bool(v) => Some(CachedConstant::Bool(v)),
            &TypedValue::Int(v) => Some(CachedConstant::Int(v)),
            &TypedValue::UInt(v) => Some(CachedConstant::UInt(v)),
            &TypedValue::Float(v) => Some(CachedConstant::from_f32(v)),
            &TypedValue::Double(v) => Some(CachedConstant::from_f64(v)),
            _ => None,
        };

        if let Some(ref key) = cache {
            if let Some(id) = self.constants.get(key) {
                return Ok(*id);
            }
        }

        let id = impl_register_constant!(self, constant)?;
        if let Some(key) = cache {
            self.constants.insert(key, id);
        }

        Ok(id)
    }

    fn visit(&mut self, graph: &Graph, index: NodeIndex<u32>) -> Result<(&'static TypeName, u32)> {
        if let Some(res) = self.results.get(&index) {
            return Ok(*res);
        }

        let args: Result<Vec<_>> =
            graph.arguments(index)
                .map(|edge| self.visit(graph, edge))
                .collect();

        let ref node = graph[index];
        let res = node.get_result(self, args?)
            .chain_err(|| ErrorKind::BuildError(node.to_string(), index.index()))?;

        self.results.insert(index, res);
        Ok(res)
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
        result.extend(
            self.imports.values()
                .map(|&(_, ref op)| op.clone())
        );

        // Memory Model
        result.push(Instruction::MemoryModel {
            addressing_model: desc::AddressingModel::Logical,
            memory_model: desc::MemoryModel::GLSL450
        });

        // Entry points
        let mut prog_io = Vec::with_capacity(
            self.inputs.len() +
            self.outputs.len()
        );
        prog_io.extend(
            self.inputs.iter()
        );
        prog_io.extend(
            self.outputs.iter()
        );

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
        let mut res: Vec<_> = Vec::with_capacity(
            (operations.len() * 4) + 5
        );

        res.push(0x07230203);
        res.push(0x00010000);
        res.push(0x000c0001);
        res.push(self.bound());
        res.push(0);

        res.extend(
            operations.into_iter()
                .flat_map(to_bytecode)
        );

        let res = res.as_slice();
        Vec::from(unsafe {
            slice::from_raw_parts(
                mem::transmute(res.as_ptr()),
                res.len() * 4
            )
        })
    }
}
