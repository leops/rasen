use std::mem;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use petgraph::graph::{NodeIndex, Graph as PetGraph};
use petgraph::algo::toposort;

use spirv_headers::ExecutionModel as ShaderType;
use spirv_headers::*;
use rspirv::binary::Assemble;
use rspirv::mr::{
    Module, ModuleHeader, BasicBlock,
    Instruction, Function, Operand
};

use graph::*;
use errors::*;
use types::{TypeName, TypedValue};

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
            (bits & 0xf_ffff_ffff_ffff) << 1
        } else {
            (bits & 0xf_ffff_ffff_ffff) | 0x10_0000_0000_0000
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
            (bits & 0x7f_ffff) << 1
        } else {
            (bits & 0x7f_ffff) | 0x80_0000
        };

        exponent -= 127 + 23;
        CachedConstant::Float(sign, exponent, mantissa)
    }
}

/// Builds the dependency graph for a list of Instructions and perform a topological sort
fn sort_instructions(unsorted: &[Instruction]) -> Result<Vec<Instruction>> {
    let mut decl_graph = PetGraph::new();

    let mut node_map = HashMap::new();
    for inst in unsorted {
        let index = decl_graph.add_node(inst);
        if let Some(id) = inst.result_id {
            node_map.insert(id, index);
        }
    }

    for inst in unsorted {
        if let Some(id) = inst.result_id {
            let self_node = &node_map[&id];

            if let Some(id) = inst.result_type {
                let other = &node_map[&id];
                decl_graph.add_edge(*other, *self_node, ());
            }

            for op in &inst.operands {
                if let Operand::IdRef(ref id) = *op {
                    let other = &node_map[id];
                    decl_graph.add_edge(*other, *self_node, ());
                }
            }
        }
    }

    match toposort(&decl_graph, None) {
        Err(_) => Err(ErrorKind::CyclicGraph.into()),
        Ok(indices) => Ok(
            indices.into_iter()
                .map(|i| {
                    let inst = decl_graph[i];
                    Instruction {
                        class: inst.class,
                        result_type: inst.result_type,
                        result_id: inst.result_id,
                        operands: inst.operands.iter()
                            .map(|op| match *op {
                                Operand::IdRef(id) => Operand::IdRef(id),
                                Operand::LiteralInt32(val) => Operand::LiteralInt32(val),
                                Operand::LiteralFloat32(val) => Operand::LiteralFloat32(val),
                                Operand::StorageClass(output) => Operand::StorageClass(output),
                                Operand::Dim(dim) => Operand::Dim(dim),
                                Operand::ImageFormat(format) => Operand::ImageFormat(format),
                                _ => panic!("unimplemented Operand::{:?}", op),
                            })
                            .collect(),
                    }
                })
                .collect()
        ),
    }
}

/// Builder struct used to define a SPIR-V module
#[derive(Debug)]
pub struct Builder {
    mod_type: ShaderType,
    counter: AtomicUsize,

    pub inputs: Vec<Word>,
    pub outputs: Vec<Word>,

    imports: HashMap<&'static str, (Word, Instruction)>,

    module: Module,
    instructions: Vec<Instruction>,

    uniform: Option<(Word, Word)>,
    uniforms: HashMap<Word, (Word, &'static TypeName)>,

    types: HashMap<&'static TypeName, Word>,
    constants: HashMap<CachedConstant, Word>,
    results: HashMap<NodeIndex<Word>, (&'static TypeName, Word)>,
}

const VOID_ID: Word = 1;
const FUNC_ID: Word = 2;
const LABEL_ID: Word = 3;
const ENTRY_ID: Word = 4;

include!(concat!(env!("OUT_DIR"), "/builder.rs"));

impl Builder {
    /// Create a new shader builder with some predefined base values
    pub fn new(mod_type: ShaderType) -> Builder {
        Builder {
            mod_type: mod_type,
            counter: AtomicUsize::new(5),

            inputs: Default::default(),
            outputs: Default::default(),

            imports: Default::default(),

            module: Module {
                capabilities: vec![
                    Instruction::new(
                        Op::Capability,
                        None, None,
                        vec![
                            Operand::Capability(Capability::Shader),
                        ]
                    ),
                ],

                memory_model: Some(
                    Instruction::new(
                        Op::MemoryModel,
                        None, None,
                        vec![
                            Operand::AddressingModel(AddressingModel::Logical),
                            Operand::MemoryModel(MemoryModel::GLSL450)
                        ]
                    )
                ),

                execution_modes: vec![
                    Instruction::new(
                        Op::ExecutionMode,
                        None, None,
                        vec![
                            Operand::IdRef(ENTRY_ID),
                            Operand::ExecutionMode(ExecutionMode::OriginUpperLeft)
                        ]
                    )
                ],

                types_global_values: vec![
                    Instruction::new(
                        Op::TypeVoid,
                        None,
                        Some(VOID_ID),
                        Vec::new()
                    ),
                    Instruction::new(
                        Op::TypeFunction,
                        None,
                        Some(FUNC_ID),
                        vec![
                            Operand::IdRef(VOID_ID),
                        ]
                    )
                ],

                .. Module::new()
            },
            instructions: Vec::new(),

            uniform: None,
            uniforms: Default::default(),

            types: Default::default(),
            constants: Default::default(),
            results: Default::default(),
        }
    }

    /// Create a new Builder and add instructions to it based on a Graph
    pub fn build(graph: &Graph, mod_type: ShaderType) -> Result<Builder> {
        if graph.has_cycle() {
            return Err(ErrorKind::CyclicGraph.into());
        }

        let mut program = Builder::new(mod_type);
        for node in graph.outputs() {
            program.visit(graph, node)?;
        }

        Ok(program)
    }

    /// Acquire a new identifier to be used in the module
    #[inline]
    pub fn get_id(&mut self) -> Word {
        self.counter.fetch_add(1, Ordering::SeqCst) as Word
    }

    fn get_uniform_block(&mut self) -> (Word, Word) {
        if let Some(res) = self.uniform {
            return res;
        }

        let ty_id = self.get_id();
        self.module.annotations.push(
            Instruction::new(
                Op::Decorate,
                None,
                None,
                vec![
                    Operand::IdRef(ty_id),
                    Operand::Decoration(Decoration::Block),
                ],
            )
        );

        let ptr_id = self.get_id();
        self.module.types_global_values.push(
            Instruction::new(
                Op::TypePointer,
                None,
                Some(ptr_id),
                vec![
                    Operand::StorageClass(StorageClass::Uniform),
                    Operand::IdRef(ty_id),
                ]
            )
        );

        let var_id = self.get_id();
        self.module.types_global_values.push(
            Instruction::new(
                Op::Variable,
                Some(ptr_id),
                Some(var_id),
                vec![
                    Operand::StorageClass(StorageClass::Uniform),
                ]
            )
        );

        let res = (ty_id, var_id);
        self.uniform = Some(res);
        res
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

    /// Get the list of inputs and outputs of this module
    pub fn get_io(&self) -> Vec<u32> {
        self.inputs.iter()
            .chain(self.outputs.iter())
            .cloned()
            .collect()
    }

    /// Import an instruction set to the module, returning its ID
    pub fn import_set(&mut self, name: &'static str) -> Word {
        if let Some(&(id, _)) = self.imports.get(&name) {
            return id;
        }

        let ext_id = self.get_id();
        self.imports.insert(name, (
            ext_id,
            Instruction::new(
                Op::ExtInstImport,
                None,
                Some(ext_id),
                vec![
                    Operand::LiteralString(name.into()),
                ]
            )
        ));

        ext_id
    }

    /// Get the ID corresponding to a Type
    pub fn register_type(&mut self, type_id: &'static TypeName) -> Word {
        if let Some(reg_id) = self.types.get(type_id) {
            return *reg_id;
        }

        let res_id = match *type_id {
            TypeName::Bool => {
                let bool_id = self.get_id();

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeBool,
                        None,
                        Some(bool_id),
                        vec![]
                    )
                );

                bool_id
            },
            TypeName::Int(is_signed) => {
                let int_id = self.get_id();

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeInt,
                        None,
                        Some(int_id),
                        vec![
                            Operand::LiteralInt32(32),
                            Operand::LiteralInt32(if is_signed { 1 } else { 0 })
                        ]
                    )
                );

                int_id
            },
            TypeName::Float(is_double) => {
                let float_id = self.get_id();

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeFloat,
                        None,
                        Some(float_id),
                        vec![
                            Operand::LiteralInt32(if is_double {
                                64
                            } else {
                                32
                            })
                        ]
                    )
                );

                float_id
            },
            TypeName::Vec(len, scalar) => {
                let float_id = self.register_type(scalar);
                let vec_id = self.get_id();

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeVector,
                        None,
                        Some(vec_id),
                        vec![
                            Operand::IdRef(float_id),
                            Operand::LiteralInt32(len),
                        ]
                    )
                );

                vec_id
            },
            TypeName::Mat(len, vec) => {
                let vec_id = self.register_type(vec);
                let mat_id = self.get_id();

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeMatrix,
                        None,
                        Some(mat_id),
                        vec![
                            Operand::IdRef(vec_id),
                            Operand::LiteralInt32(len),
                        ]
                    )
                );

                mat_id
            },
            TypeName::Sampler(sampled_type, dimensionality) => {
                let sample_id = self.register_type(sampled_type);
                let image_id = self.get_id();
                let sampler_id = self.get_id();

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeImage,
                        None,
                        Some(image_id),
                        vec![
                            Operand::IdRef(sample_id),
                            Operand::Dim(dimensionality),
                            Operand::LiteralInt32(0),
                            Operand::LiteralInt32(0),
                            Operand::LiteralInt32(0),
                            Operand::LiteralInt32(1),
                            Operand::ImageFormat(ImageFormat::Unknown),
                        ]
                    )
                );

                self.module.types_global_values.push(
                    Instruction::new(
                        Op::TypeSampledImage,
                        None,
                        Some(sampler_id),
                        vec![
                            Operand::IdRef(image_id),
                        ]
                    )
                );

                sampler_id
            },
        };

        self.types.insert(type_id, res_id);
        res_id
    }

    /// Add a new constant to the module, returning its ID
    pub fn register_constant(&mut self, constant: &TypedValue) -> Result<u32> {
        let cache = match *constant {
            TypedValue::Bool(v) => Some(CachedConstant::Bool(v)),
            TypedValue::Int(v) => Some(CachedConstant::Int(v)),
            TypedValue::UInt(v) => Some(CachedConstant::UInt(v)),
            TypedValue::Float(v) => Some(CachedConstant::from_f32(v)),
            TypedValue::Double(v) => Some(CachedConstant::from_f64(v)),
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

    pub fn register_uniform(&mut self, location: u32, type_id: &'static TypeName) -> Word {
        let (_, var_id) = self.get_uniform_block();

        let ty_id = self.register_type(type_id);
        self.uniforms.insert(location, (ty_id, type_id));

        var_id
    }

    pub fn push_annotation(&mut self, inst: Instruction) {
        self.module.annotations.push(inst);
    }

    pub fn push_declaration(&mut self, inst: Instruction) {
        self.module.types_global_values.push(inst);
    }

    pub fn push_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    fn visit(&mut self, graph: &Graph, index: NodeIndex<u32>) -> Result<(&'static TypeName, u32)> {
        if let Some(res) = self.results.get(&index) {
            return Ok(*res);
        }

        let args: Result<Vec<_>> = {
            graph.arguments(index)
                .map(|edge| self.visit(graph, edge))
                .collect()
        };

        let node = &graph[index];
        let res = {
            node.get_result(self, args?)
                .chain_err(|| {
                    ErrorKind::BuildError(node.to_string(), index.index())
                })?
        };

        self.results.insert(index, res);
        Ok(res)
    }

    /// Build the module, returning a list of instructions
    pub fn finalize(mut self) -> Result<Module> {
        let mut uniforms: Vec<(Word, Word, &'static TypeName)> = {
            self.uniforms.iter()
                .map(|(k, &(a, b))| (*k, a, b))
                .collect()
        };

        uniforms.sort_by_key(|&(k, _, _)| k);

        Ok(Module {
            header: Some(
                ModuleHeader {
                    magic_number: MAGIC_NUMBER,
                    version: (MAJOR_VERSION << 16) | (MAJOR_VERSION << 8),
                    generator: 0xffff_0009,
                    bound: self.bound(),
                    reserved_word: 0,
                },
            ),

            ext_inst_imports:
                self.imports.into_iter()
                    .map(|(_, (_, op))| op)
                    .collect(),

            entry_points: vec![
                Instruction::new(
                    Op::EntryPoint,
                    None, None,
                    {
                        let mut res = Vec::with_capacity(
                            self.inputs.len() +
                            self.outputs.len() +
                            3
                        );

                        res.push(Operand::ExecutionModel(self.mod_type));
                        res.push(Operand::IdRef(ENTRY_ID));
                        res.push(Operand::LiteralString("main".into()));

                        res.extend(
                            self.inputs.into_iter().map(Operand::IdRef)
                        );
                        res.extend(
                            self.outputs.into_iter().map(Operand::IdRef)
                        );

                        res
                    }
                )
            ],

            annotations: {
                let mut res = self.module.annotations;

                if let Some((ty_id, _)) = self.uniform {
                    let mut offset = 0;
                    for &(location, _, type_id) in &uniforms {
                        res.push(
                            Instruction::new(
                                Op::MemberDecorate,
                                None, None,
                                vec![
                                    Operand::IdRef(ty_id),
                                    Operand::LiteralInt32(location),
                                    Operand::Decoration(Decoration::Offset),
                                    Operand::LiteralInt32(offset),
                                ]
                            )
                        );

                        offset += type_id.size();
                    }
                }

                res
            },

            types_global_values: {
                let mut declarations = Vec::with_capacity(
                    self.module.types_global_values.len() + 1
                );

                // Uniforms
                if let Some((ty_id, _)) = self.uniform {
                    declarations.push(
                        Instruction::new(
                            Op::TypeStruct,
                            None,
                            Some(ty_id),
                            uniforms.into_iter()
                                .map(|(_, v, _)| Operand::IdRef(v))
                                .collect()
                        )
                    );
                }

                // Declarations
                declarations.append(&mut self.module.types_global_values);
                sort_instructions(&declarations)?
            },

            // Functions
            functions: vec![
                Function {
                    def: Some(
                        Instruction::new(
                            Op::Function,
                            Some(VOID_ID),
                            Some(ENTRY_ID),
                            vec![
                                Operand::FunctionControl(FunctionControl::empty()),
                                Operand::IdRef(FUNC_ID),
                            ]
                        )
                    ),
                    end: Some(
                        Instruction::new(
                            Op::FunctionEnd,
                            None, None,
                            Vec::new()
                        )
                    ),
                    parameters: Vec::new(),
                    basic_blocks: vec![
                        BasicBlock {
                            label: Some(
                                Instruction::new(
                                    Op::Label,
                                    None,
                                    Some(LABEL_ID),
                                    Vec::new()
                                )
                            ),
                            instructions: {
                                let mut res = self.instructions;
                                res.push(
                                    Instruction::new(
                                        Op::Return,
                                        None, None,
                                        Vec::new()
                                    )
                                );

                                res
                            }
                        }
                    ],
                },
            ],

            .. self.module
        })
    }

    /// Get the instructions of the module in binary form
    pub fn into_bytecode(self) -> Result<Vec<u8>> {
        let mut res = self.finalize()?.assemble();
        let ptr = res.as_mut_ptr();
        let len = res.len().checked_mul(4).ok_or("Integer overflow")?;
        let cap = res.capacity().checked_mul(4).ok_or("Integer overflow")?;

        Ok(unsafe {
            mem::forget(res);
            Vec::from_raw_parts(
                mem::transmute(ptr),
                len, cap,
            )
        })
    }
}
