use std::iter;

use petgraph::graph::{NodeIndex};
use fnv::FnvHashMap as HashMap;

use spirv_headers::*;
use rspirv::mr::{BasicBlock, Instruction, Function, Operand};

use errors::*;
use module::{FunctionRef};
use types::{TypeName, TypedValue};
use super::Builder;
use super::module::{ModuleBuilder, VOID_ID};

pub struct FunctionBuilder<'a> {
    module: &'a mut ModuleBuilder,
    results: HashMap<NodeIndex<Word>, (&'static TypeName, Word)>,

    id: Word,
    args: Vec<(&'static TypeName, Instruction)>,
    res: Option<&'static TypeName>,
    instructions: Vec<Instruction>,
}

impl<'a> FunctionBuilder<'a> {
    pub fn new(module: &'a mut ModuleBuilder) -> FunctionBuilder<'a> {
        let id = module.get_id();
        FunctionBuilder {
            module,
            results: Default::default(),

            id,
            args: Vec::new(),
            res: None,
            instructions: vec![
                Instruction::new(
                    Op::Return,
                    None, None,
                    Vec::new(),
                )
            ],
        }
    }

    pub fn build(self) {
        let FunctionBuilder { module, id, args, instructions, res, .. } = self;

        let label_id = module.get_id();
        let result_type = if let Some(ty) = res {
            module.register_type(ty)
        } else {
            VOID_ID
        };

        let (args_ty, parameters): (Vec<_>, _) = args.into_iter().unzip();

        let func_type = module.get_id();
        let func_def = Instruction::new(
            Op::TypeFunction,
            None,
            Some(func_type),
            iter::once(result_type)
                .chain(
                    args_ty.iter()
                        .map(|ty| module.register_type(ty))
                )
                .map(Operand::IdRef)
                .collect(),
        );

        module.module.types_global_values.push(func_def);

        module.functions.push((
            id, args_ty, res,
            Function {
                def: Some(
                    Instruction::new(
                        Op::Function,
                        Some(result_type),
                        Some(id),
                        vec![
                            Operand::FunctionControl(FunctionControl::empty()),
                            Operand::IdRef(func_type),
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
                parameters,
                basic_blocks: vec![
                    BasicBlock {
                        label: Some(
                            Instruction::new(
                                Op::Label,
                                None,
                                Some(label_id),
                                Vec::new()
                            )
                        ),
                        instructions,
                    }
                ],
            },
        ));
    }
}

impl<'a> Builder for FunctionBuilder<'a> {
    fn get_id(&mut self) -> Word {
        self.module.get_id()
    }

    fn import_set(&mut self, name: &'static str) -> Word {
        self.module.import_set(name)
    }

    fn register_type(&mut self, type_id: &'static TypeName) -> Word {
        self.module.register_type(type_id)
    }

    fn register_constant(&mut self, constant: &TypedValue) -> Result<u32> {
        self.module.register_constant(constant)
    }

    fn register_uniform(&mut self, location: u32, type_id: &'static TypeName) -> (Word, Word) {
        self.module.register_uniform(location, type_id)
    }

    fn push_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst)
    }

    fn push_declaration(&mut self, inst: Instruction) {
        self.module.push_declaration(inst)
    }
    
    fn push_output(&mut self, id: Word) {
        self.module.push_output(id)
    }

    fn push_input(&mut self, id: Word) {
        self.module.push_input(id)
    }

    fn push_annotation(&mut self, inst: Instruction) {
        self.module.push_annotation(inst)
    }

    fn push_debug(&mut self, inst: Instruction) {
        self.module.push_debug(inst)
    }

    fn push_parameter(&mut self, location: u32, ty: &'static TypeName, inst: Instruction) -> Result<()> {
        let index = location as usize;
        while self.args.len() <= index {
            self.args.push((
                TypeName::VOID,
                Instruction::new(
                    Op::FunctionParameter,
                    None, None,
                    Vec::new(),
                )
            ));
        }

        self.args[index] = (ty, inst);

        Ok(())
    }

    fn set_return(&mut self, ty: &'static TypeName, inst: Instruction) -> Result<()> {
        let last = self.instructions.last_mut().expect("instructions should not be empty");
        *last = inst;
        
        self.res = Some(ty);

        Ok(())
    }

    fn get_result(&self, index: &NodeIndex<u32>) -> Option<(&'static TypeName, u32)> {
        self.results.get(index).cloned()
    }

    fn set_result(&mut self, index: NodeIndex<u32>, res: (&'static TypeName, u32)) {
        self.results.insert(index, res);
    }

    fn get_function(&self, index: FunctionRef) -> Option<(Word, &[&'static TypeName], Option<&'static TypeName>)> {
        self.module.get_function(index)
    }
}
