//! Definition and implentations of all the graph operations

use std::fmt;
use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId,
};
use spirv_utils::instruction::*;

use super::Module;
use super::types::*;
use errors::*;
use operations;

include!(concat!(env!("OUT_DIR"), "/node.rs"));

impl Node {
    /// Insert this Node into a Program
    pub fn get_result(&self, module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
        use glsl::GLSL::*;

        macro_rules! impl_glsl_call {
            ( $function:ident, $argc:expr ) => {
                {
                    if args.len() != $argc {
                        Err(ErrorKind::WrongArgumentsCount(args.len(), $argc))?;
                    }

                    let ext_id = module.import_set(String::from("GLSL.std.450"));

                    let (res_ty, _) = args[0];
                    let res_type = module.register_type(res_ty);

                    let args: ::std::vec::Vec<_> = args.into_iter()
                        .map(|(_, rid)| Id(rid))
                        .collect();

                    let result_id = module.get_id();

                    module.instructions.push(Instruction::ExtInst {
                        result_type: TypeId(res_type),
                        result_id: ResultId(result_id),
                        set: ValueId(ext_id),
                        instruction: $function as u32,
                        operands: args.into_boxed_slice(),
                    });

                    Ok((res_ty, result_id))
                }
            };
        }

        match self {
            &Node::Output(location, attr_type) => {
                if args.len() != 1 {
                    Err(ErrorKind::WrongArgumentsCount(args.len(), 1))?;
                }

                let (arg_type, arg_value) = args[0];
                if arg_type != attr_type {
                    Err(ErrorKind::BadArguments(Box::new([ arg_type ])))?;
                }

                let type_id = module.register_type(attr_type);
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
                    obj: ValueId(arg_value),
                    memory_access: desc::MemoryAccess::empty()
                });

                Ok((attr_type, var_id))
            },

            &Node::Input(location, attr_type) => {
                let type_id = module.register_type(attr_type);

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

                Ok((attr_type, res_id))
            },

            &Node::Constant(ref const_type) => Ok((const_type.to_type_name(), module.register_constant(const_type)?)),

            &Node::Construct(output_type) => {
                let type_id = module.register_type(output_type);
                let res_id = module.get_id();

                module.instructions.push(Instruction::CompositeConstruct {
                    result_type: TypeId(type_id),
                    result_id: ResultId(res_id),
                    fields: match output_type {
                        &TypeName::Vec(size, data_type) => {
                            if args.len() != size as usize {
                                Err(ErrorKind::WrongArgumentsCount(args.len(), size as usize))?;
                            }

                            let res: Result<Vec<_>> =
                                args.into_iter()
                                    .map(|(ty, val)| {
                                        if ty != data_type {
                                            Err(ErrorKind::BadArguments(Box::new([ ty ])))?;
                                        }

                                        Ok(ValueId(val))
                                    })
                                    .collect();

                            res?.into_boxed_slice()
                        },
                        _ => Err(ErrorKind::BadArguments(Box::new([ output_type ])))?,
                    },
                });

                Ok((output_type, res_id))
            },

            &Node::Extract(index) => {
                if args.len() != 1 {
                    Err(ErrorKind::WrongArgumentsCount(args.len(), 1))?;
                }

                let (arg_type, arg_value) = args[0];
                match arg_type {
                    &TypeName::Vec(len, data_ty) => {
                        if index >= len {
                            Err(format!("Index out of bounds ({} >= {})", index, len))?;
                        }

                        let type_id = module.register_type(data_ty);
                        let res_id = module.get_id();

                        module.instructions.push(Instruction::CompositeExtract {
                            result_type: TypeId(type_id),
                            result_id: ResultId(res_id),
                            obj: ValueId(arg_value),
                            indices: Box::new([ index ]),
                        });

                        Ok((data_ty, res_id))
                    },
                    _ => Err(ErrorKind::BadArguments(Box::new([ arg_type ])))?,
                }
            },

            &Node::Add => operations::add(module, args),
            &Node::Substract => operations::substract(module, args),
            &Node::Multiply => operations::multiply(module, args),
            &Node::Divide => operations::divide(module, args),
            &Node::Modulus => operations::modulus(module, args),
            &Node::Dot => operations::dot(module, args),

            &Node::Clamp => operations::clamp(module, args),
            &Node::Mix => operations::mix(module, args),

            &Node::Normalize => impl_glsl_call!(Normalize, 1),
            &Node::Cross => impl_glsl_call!(Cross, 2),

            &Node::Pow => impl_glsl_call!(Pow, 2),
            &Node::Floor => impl_glsl_call!(Floor, 1),
            &Node::Ceil => impl_glsl_call!(Ceil, 1),
            &Node::Round => impl_glsl_call!(Round, 1),

            &Node::Sin => operations::sin(module, args),
            &Node::Cos => operations::cos(module, args),
            &Node::Tan => operations::tan(module, args),

            &Node::Min => operations::min(module, args),
            &Node::Max => operations::max(module, args),

            &Node::Length => operations::length(module, args),
            &Node::Distance => operations::distance(module, args),
            &Node::Reflect => operations::reflect(module, args),
            &Node::Refract => operations::refract(module, args),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
