use spirv_utils::instruction::*;
use spirv_utils::desc::{
    ResultId, TypeId, ValueId,
};

use module::Module;
use types::*;
use errors::*;

#[inline]
fn imul(module: &mut Module, res_type: &'static TypeName, res_id: u32, lhs: u32, rhs: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::IMul {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        lhs: ValueId(lhs),
        rhs: ValueId(rhs),
    });
}
#[inline]
fn fmul(module: &mut Module, res_type: &'static TypeName, res_id: u32, lhs: u32, rhs: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::FMul {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        lhs: ValueId(lhs),
        rhs: ValueId(rhs),
    });
}
#[inline]
fn vector_times_scalar(module: &mut Module, res_type: &'static TypeName, res_id: u32, vector: u32, scalar: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::VectorTimesScalar {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        vector: ValueId(vector),
        scalar: ValueId(scalar),
    });
}
#[inline]
fn matrix_times_scalar(module: &mut Module, res_type: &'static TypeName, res_id: u32, matrix: u32, scalar: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::MatrixTimesScalar {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        matrix: ValueId(matrix),
        scalar: ValueId(scalar),
    });
}

#[inline]
pub fn multiply(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    let res_id = module.get_id();

    let res_type = match (l_type, r_type) {
        _ if l_type == r_type && r_type.is_integer() => {
            imul(module, l_type, res_id, l_value, r_value);
            l_type
        },
        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => {
            imul(module, l_type, res_id, l_value, r_value);
            l_type
        },

        _ if l_type == r_type && l_type.is_float() => {
            fmul(module, l_type, res_id, l_value, r_value);
            l_type
        },
        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_float() => {
            fmul(module, l_type, res_id, l_value, r_value);
            l_type
        },

        (&Vec(_, v_scalar), t_scalar) if t_scalar == v_scalar && t_scalar.is_float() => {
            vector_times_scalar(module, l_type, res_id, r_value, l_value);
            l_type
        },
        (t_scalar, &Vec(_, v_scalar)) if t_scalar == v_scalar && t_scalar.is_float() => {
            vector_times_scalar(module, r_type, res_id, l_value, r_value);
            r_type
        },

        (&Mat(_, m_scalar), t_scalar) if t_scalar == m_scalar && t_scalar.is_float() => {
            matrix_times_scalar(module, l_type, res_id, l_value, r_value);
            l_type
        },
        (t_scalar, &Mat(_, m_scalar)) if t_scalar == m_scalar && t_scalar.is_float() => {
            matrix_times_scalar(module, r_type, res_id, r_value, l_value);
            r_type
        },

        (&Vec(v_len, l_scalar), &Mat(m_len, r_scalar)) if v_len == m_len && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_type);

            module.instructions.push(Instruction::VectorTimesMatrix {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                vector: ValueId(l_value),
                matrix: ValueId(r_value),
            });

            l_type
        },
        (&Mat(m_len, l_scalar), &Vec(v_len, r_scalar)) if v_len == m_len && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_type);

            module.instructions.push(Instruction::MatrixTimesVector {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                matrix: ValueId(l_value),
                vector: ValueId(r_value),
            });

            l_type
        },

        (&Mat(l_len, l_scalar), &Mat(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_type);

            module.instructions.push(Instruction::MatrixTimesMatrix {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                lhs: ValueId(l_value),
                rhs: ValueId(r_value),
            });

            l_type
        },

        _ => return Err(ErrorKind::BadArguments(Box::new([ l_type, r_type ])).into()),
    };

    Ok((res_type, res_id))
}

#[inline]
pub fn dot(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_scalar);

            let result_id = module.get_id();

            module.instructions.push(Instruction::Dot {
                result_type: TypeId(res_type),
                result_id: ResultId(result_id),
                lhs: ValueId(l_value),
                rhs: ValueId(r_value),
            });

            Ok((l_scalar, result_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([ l_type, r_type ])))?,
    }
}

macro_rules! impl_math_op {
    ( $name:ident, $node:ident, $( $opcode:ident ),* ) => {
        #[inline]
        pub fn $name(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            if args.len() != 2 {
                Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
            }

            let result_id = module.get_id();

            let (l_type, l_value) = args[0];
            let (r_type, r_value) = args[1];

            macro_rules! match_types {
                ( $uopcode:ident, $sopcode:ident, $fopcode:ident ) => {
                    match (l_type, r_type) {
                        _ if l_type == r_type && r_type.is_signed() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$sopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_signed() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$sopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },

                        _ if l_type == r_type && r_type.is_integer() && !r_type.is_signed() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$uopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$uopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },

                        _ if l_type == r_type && r_type.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },

                        _ => Err(ErrorKind::BadArguments(Box::new([ l_type, r_type ])))?,
                    }
                };
                ( $iopcode:ident, $fopcode:ident ) => {
                    match (l_type, r_type) {
                        _ if l_type == r_type && r_type.is_integer() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$iopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$iopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });

                            l_type
                        },

                        _ if l_type == r_type && r_type.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(r_value),
                                rhs: ValueId(l_value),
                            });

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && l_scalar.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(r_value),
                                rhs: ValueId(l_value),
                            });

                            l_type
                        },

                        _ => return Err(ErrorKind::BadArguments(Box::new([ l_type, r_type ])).into()),
                    }
                };
            }

            let res_type = match_types!( $( $opcode ),* );
            Ok((res_type, result_id))
        }
    };
}

impl_math_op!(add, Add, IAdd, FAdd);
impl_math_op!(substract, Substract, ISub, FSub);
impl_math_op!(divide, Divide, UDiv, SDiv, FDiv);
impl_math_op!(modulus, Modulus, UMod, SMod, FMod);
