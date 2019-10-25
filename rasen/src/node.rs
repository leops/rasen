//! Definition and implentations of all the graph operations

use rspirv::mr::{Instruction, Operand};
use spirv_headers::*;
use std::{fmt, iter};

use builder::Builder;
use errors::*;
use module::FunctionRef;
use operations;
use types::*;

include!(concat!(env!("OUT_DIR"), "/node.rs"));

impl Node {
    /// Insert this Node into a Program
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn get_result(
        &self,
        module: &mut impl Builder,
        args: Vec<(&'static TypeName, u32)>,
    ) -> Result<(&'static TypeName, u32)> {
        // use spirv_headers::GLOp::*;

        macro_rules! impl_glsl_call {
            ( $function:ident, $argc:expr ) => {{
                if args.len() != $argc {
                    bail!(ErrorKind::WrongArgumentsCount(args.len(), $argc));
                }

                let ext_id = module.import_set("GLSL.std.450");

                let (res_ty, _) = args[0];
                let res_type = module.register_type(res_ty);

                let mut operands = Vec::with_capacity($argc + 2);
                operands.push(Operand::IdRef(ext_id));
                operands.push(Operand::LiteralExtInstInteger(
                    ::spirv_headers::GLOp::$function as u32,
                ));
                operands.extend(args.into_iter().map(|(_, rid)| Operand::IdRef(rid)));

                let result_id = module.get_id();

                module.push_instruction(Instruction::new(
                    Op::ExtInst,
                    Some(res_type),
                    Some(result_id),
                    operands,
                ));

                Ok((res_ty, result_id))
            }};
        }

        match *self {
            Node::Output(location, attr_type, ref name) => {
                if args.len() != 1 {
                    bail!(ErrorKind::WrongArgumentsCount(args.len(), 1));
                }

                let (arg_type, arg_value) = args[0];
                if arg_type != attr_type {
                    bail!(ErrorKind::BadArguments(Box::new([arg_type])));
                }

                let type_id = module.register_type(attr_type);
                let ptr_type = module.get_id();

                module.push_declaration(Instruction::new(
                    Op::TypePointer,
                    None,
                    Some(ptr_type),
                    vec![
                        Operand::StorageClass(StorageClass::Output),
                        Operand::IdRef(type_id),
                    ],
                ));

                let var_id = module.get_id();

                module.push_output(var_id);

                module.push_declaration(Instruction::new(
                    Op::Variable,
                    Some(ptr_type),
                    Some(var_id),
                    vec![Operand::StorageClass(StorageClass::Output)],
                ));

                module.push_annotation(Instruction::new(
                    Op::Decorate,
                    None,
                    None,
                    vec![
                        Operand::IdRef(var_id),
                        Operand::Decoration(Decoration::Location),
                        Operand::LiteralInt32(location),
                    ],
                ));

                name.decorate_variable(module, var_id);

                module.push_instruction(Instruction::new(
                    Op::Store,
                    None,
                    None,
                    vec![
                        Operand::IdRef(var_id),
                        Operand::IdRef(arg_value),
                        Operand::MemoryAccess(MemoryAccess::empty()),
                    ],
                ));

                Ok((attr_type, var_id))
            }

            Node::Input(location, attr_type, ref name) => {
                let type_id = module.register_type(attr_type);

                let ptr_type = module.get_id();

                module.push_declaration(Instruction::new(
                    Op::TypePointer,
                    None,
                    Some(ptr_type),
                    vec![
                        Operand::StorageClass(StorageClass::Input),
                        Operand::IdRef(type_id),
                    ],
                ));

                let var_id = module.get_id();

                module.push_input(var_id);

                module.push_declaration(Instruction::new(
                    Op::Variable,
                    Some(ptr_type),
                    Some(var_id),
                    vec![Operand::StorageClass(StorageClass::Input)],
                ));

                module.push_annotation(Instruction::new(
                    Op::Decorate,
                    None,
                    None,
                    vec![
                        Operand::IdRef(var_id),
                        Operand::Decoration(Decoration::Location),
                        Operand::LiteralInt32(location),
                    ],
                ));

                name.decorate_variable(module, var_id);

                let res_id = module.get_id();

                module.push_instruction(Instruction::new(
                    Op::Load,
                    Some(type_id),
                    Some(res_id),
                    vec![
                        Operand::IdRef(var_id),
                        Operand::MemoryAccess(MemoryAccess::empty()),
                    ],
                ));

                Ok((attr_type, res_id))
            }

            Node::Uniform(location, attr_type, ref name) => {
                let type_id = module.register_type(attr_type);
                let ptr_type = module.register_type(attr_type.as_ptr(true));

                if let attr_type @ &TypeName::Sampler(_, _) = attr_type {
                    let var_id = module.get_id();
                    module.push_declaration(Instruction::new(
                        Op::Variable,
                        Some(ptr_type),
                        Some(var_id),
                        vec![Operand::StorageClass(StorageClass::Uniform)],
                    ));

                    module.push_annotation(Instruction::new(
                        Op::Decorate,
                        None,
                        None,
                        vec![
                            Operand::IdRef(var_id),
                            Operand::Decoration(Decoration::Location),
                            Operand::LiteralInt32(location),
                        ],
                    ));

                    name.decorate_variable(module, var_id);

                    let res_id = module.get_id();
                    module.push_instruction(Instruction::new(
                        Op::Load,
                        Some(type_id),
                        Some(res_id),
                        vec![
                            Operand::IdRef(var_id),
                            Operand::MemoryAccess(MemoryAccess::empty()),
                        ],
                    ));

                    Ok((attr_type, res_id))
                } else {
                    let (struct_id, var_id) = module.register_uniform(location, attr_type);
                    let index_id = module.register_constant(&TypedValue::Int(location as i32))?;

                    name.decorate_member(module, struct_id, location);

                    if let TypeName::Mat(_, vec) = attr_type {
                        module.push_annotation(Instruction::new(
                            Op::MemberDecorate,
                            None,
                            None,
                            vec![
                                Operand::IdRef(struct_id),
                                Operand::LiteralInt32(location),
                                Operand::Decoration(Decoration::MatrixStride),
                                Operand::LiteralInt32(vec.size()),
                            ],
                        ));
                        module.push_annotation(Instruction::new(
                            Op::MemberDecorate,
                            None,
                            None,
                            vec![
                                Operand::IdRef(struct_id),
                                Operand::LiteralInt32(location),
                                Operand::Decoration(Decoration::ColMajor),
                            ],
                        ));
                    }

                    let chain_id = module.get_id();
                    module.push_instruction(Instruction::new(
                        Op::AccessChain,
                        Some(ptr_type),
                        Some(chain_id),
                        vec![Operand::IdRef(var_id), Operand::IdRef(index_id)],
                    ));

                    let res_id = module.get_id();
                    module.push_instruction(Instruction::new(
                        Op::Load,
                        Some(type_id),
                        Some(res_id),
                        vec![
                            Operand::IdRef(chain_id),
                            Operand::MemoryAccess(MemoryAccess::empty()),
                        ],
                    ));

                    Ok((attr_type, res_id))
                }
            }

            Node::Constant(ref const_type) => Ok((
                const_type.to_type_name(),
                module.register_constant(const_type)?,
            )),

            Node::Call(index) => {
                let (result, args) = {
                    let (func_id, args_type, result_type) =
                        if let Some(res) = module.get_function(index) {
                            res
                        } else {
                            bail!(ErrorKind::MissingFunction(index))
                        };

                    (
                        result_type,
                        iter::once(Ok(Operand::IdRef(func_id)))
                            .chain(args.into_iter().zip(args_type).map(|((val, id), arg)| {
                                if val == *arg {
                                    Ok(Operand::IdRef(id))
                                } else {
                                    bail!(ErrorKind::BadArguments(Box::new([val])));
                                }
                            }))
                            .collect::<Result<_>>()?,
                    )
                };

                let type_id = result.map(|ty| module.register_type(ty));

                let res_id = module.get_id();
                module.push_instruction(Instruction::new(
                    Op::FunctionCall,
                    type_id,
                    Some(res_id),
                    args,
                ));

                Ok((result.unwrap_or(TypeName::VOID), res_id))
            }

            Node::Parameter(location, attr_type) => {
                let type_id = module.register_type(attr_type);

                let res_id = module.get_id();
                module.push_parameter(
                    location,
                    attr_type,
                    Instruction::new(
                        Op::FunctionParameter,
                        Some(type_id),
                        Some(res_id),
                        Vec::new(),
                    ),
                )?;

                Ok((attr_type, res_id))
            }

            Node::Return => {
                if args.len() != 1 {
                    bail!(ErrorKind::WrongArgumentsCount(args.len(), 1));
                }

                let (arg_type, arg_value) = args[0];

                module.set_return(
                    arg_type,
                    Instruction::new(Op::ReturnValue, None, None, vec![Operand::IdRef(arg_value)]),
                )?;

                Ok((arg_type, arg_value))
            }

            Node::Loop(cond, body) => operations::loop_(cond, body, module, args),

            Node::Construct(output_type) => {
                let type_id = module.register_type(output_type);
                let res_id = module.get_id();

                module.push_instruction(Instruction::new(
                    Op::CompositeConstruct,
                    Some(type_id),
                    Some(res_id),
                    match *output_type {
                        TypeName::Vec(size, data_type) => {
                            if args.len() != size as usize {
                                bail!(ErrorKind::WrongArgumentsCount(args.len(), size as usize));
                            }

                            let res: Result<Vec<_>> = {
                                args.into_iter()
                                    .map(|(ty, val)| {
                                        if ty != data_type {
                                            bail!(ErrorKind::BadArguments(Box::new([ty])));
                                        }

                                        Ok(Operand::IdRef(val))
                                    })
                                    .collect()
                            };

                            res?
                        }
                        TypeName::Mat(size, vec_type) => {
                            if args.len() != size as usize {
                                bail!(ErrorKind::WrongArgumentsCount(args.len(), size as usize));
                            }

                            let res: Result<Vec<_>> = {
                                args.into_iter()
                                    .map(|(ty, val)| {
                                        if ty != vec_type {
                                            bail!(ErrorKind::BadArguments(Box::new([ty])));
                                        }

                                        Ok(Operand::IdRef(val))
                                    })
                                    .collect()
                            };

                            res?
                        }
                        _ => bail!(ErrorKind::BadArguments(Box::new([output_type]))),
                    },
                ));

                Ok((output_type, res_id))
            }

            Node::Extract(index) => {
                if args.len() != 1 {
                    bail!(ErrorKind::WrongArgumentsCount(args.len(), 1));
                }

                let (arg_type, arg_value) = args[0];
                match *arg_type {
                    TypeName::Vec(len, data_ty) => {
                        if index >= len {
                            bail!(ErrorKind::IndexOutOfBound(index, len));
                        }

                        let type_id = module.register_type(data_ty);
                        let res_id = module.get_id();

                        module.push_instruction(Instruction::new(
                            Op::CompositeExtract,
                            Some(type_id),
                            Some(res_id),
                            vec![Operand::IdRef(arg_value), Operand::LiteralInt32(index)],
                        ));

                        Ok((data_ty, res_id))
                    }
                    _ => bail!(ErrorKind::BadArguments(Box::new([arg_type]))),
                }
            }

            Node::Add => operations::add(module, args),
            Node::Subtract => operations::subtract(module, args),
            Node::Multiply => operations::multiply(module, &args),
            Node::Divide => operations::divide(module, args),
            Node::Modulus => operations::modulus(module, args),
            Node::Dot => operations::dot(module, &args),

            Node::Clamp => operations::clamp(module, args),
            Node::Mix => operations::mix(module, args),

            Node::Normalize => impl_glsl_call!(Normalize, 1),
            Node::Cross => impl_glsl_call!(Cross, 2),

            Node::Pow => impl_glsl_call!(Pow, 2),
            Node::Sqrt => impl_glsl_call!(Sqrt, 1),
            Node::Log => impl_glsl_call!(Log, 1),
            Node::Floor => impl_glsl_call!(Floor, 1),
            Node::Ceil => impl_glsl_call!(Ceil, 1),
            Node::Round => impl_glsl_call!(Round, 1),
            Node::Abs => impl_glsl_call!(FAbs, 1),
            Node::Step => impl_glsl_call!(Step, 2),
            Node::Smoothstep => impl_glsl_call!(SmoothStep, 3),
            Node::Inverse => impl_glsl_call!(MatrixInverse, 1),

            Node::Sin => operations::sin(module, args),
            Node::Cos => operations::cos(module, args),
            Node::Tan => operations::tan(module, args),

            Node::Min => operations::min(module, args),
            Node::Max => operations::max(module, args),

            Node::Length => operations::length(module, args),
            Node::Distance => operations::distance(module, &args),
            Node::Reflect => operations::reflect(module, &args),
            Node::Refract => operations::refract(module, &args),

            Node::Sample => operations::sample(module, &args),

            Node::Equal => operations::eq(module, &args),
            Node::NotEqual => operations::ne(module, &args),
            Node::Greater => operations::gt(module, &args),
            Node::GreaterEqual => operations::gte(module, &args),
            Node::Less => operations::lt(module, &args),
            Node::LessEqual => operations::lte(module, &args),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Name information for a variable
/// Can be a well-known builtin, a custom string, or nothing
/// (the variable is only adressed by its location)
#[derive(Debug)]
pub enum VariableName {
    BuiltIn(BuiltIn),
    Named(String),
    None,
}

impl VariableName {
    pub(crate) fn decorate_variable(&self, module: &mut impl Builder, var_id: Word) {
        match *self {
            VariableName::BuiltIn(built_in) => {
                module.push_annotation(Instruction::new(
                    Op::Decorate,
                    None,
                    None,
                    vec![
                        Operand::IdRef(var_id),
                        Operand::Decoration(Decoration::BuiltIn),
                        Operand::BuiltIn(built_in),
                    ],
                ));
            }

            VariableName::Named(ref name) => {
                module.push_debug(Instruction::new(
                    Op::Name,
                    None,
                    None,
                    vec![Operand::IdRef(var_id), Operand::LiteralString(name.clone())],
                ));
            }

            VariableName::None => {}
        }
    }

    fn decorate_member(&self, module: &mut impl Builder, var_id: Word, offset: Word) {
        match *self {
            VariableName::BuiltIn(built_in) => {
                module.push_annotation(Instruction::new(
                    Op::MemberDecorate,
                    None,
                    None,
                    vec![
                        Operand::IdRef(var_id),
                        Operand::LiteralInt32(offset),
                        Operand::Decoration(Decoration::BuiltIn),
                        Operand::BuiltIn(built_in),
                    ],
                ));
            }

            VariableName::Named(ref name) => {
                module.push_debug(Instruction::new(
                    Op::MemberName,
                    None,
                    None,
                    vec![
                        Operand::IdRef(var_id),
                        Operand::LiteralInt32(offset),
                        Operand::LiteralString(name.clone()),
                    ],
                ));
            }

            VariableName::None => {}
        }
    }
}
