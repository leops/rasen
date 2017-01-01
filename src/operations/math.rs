use spirv_utils::instruction::*;
use spirv_utils::desc::{
    ResultId, TypeId, ValueId,
};

use module::Module;
use types::*;

fn imul(module: &mut Module, res_type: TypeName, res_id: u32, lhs: u32, rhs: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::IMul {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        lhs: ValueId(lhs),
        rhs: ValueId(rhs),
    });
}
fn fmul(module: &mut Module, res_type: TypeName, res_id: u32, lhs: u32, rhs: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::FMul {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        lhs: ValueId(lhs),
        rhs: ValueId(rhs),
    });
}
fn vector_times_scalar(module: &mut Module, res_type: TypeName, res_id: u32, vector: u32, scalar: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::VectorTimesScalar {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        vector: ValueId(vector),
        scalar: ValueId(scalar),
    });
}
fn matrix_times_scalar(module: &mut Module, res_type: TypeName, res_id: u32, matrix: u32, scalar: u32) {
    let res_type = module.register_type(res_type);

    module.instructions.push(Instruction::MatrixTimesScalar {
        result_type: TypeId(res_type),
        result_id: ResultId(res_id),
        matrix: ValueId(matrix),
        scalar: ValueId(scalar),
    });
}

pub fn multiply(module: &mut Module, args: Vec<(TypeName, u32)>) -> Result<u32, &'static str> {
    use types::TypeName::*;

    if args.len() != 2 {
        return Err("Wrong number of arguments for Multiply");
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    let res_id = module.get_id();

    match (l_type, r_type) {
        _ if l_type == r_type && r_type.is_integer() => imul(module, l_type, res_id, l_value, r_value),
        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && r_scalar.is_integer() => imul(module, l_type, res_id, l_value, r_value),

        _ if l_type == r_type && l_type.is_float() => fmul(module, l_type, res_id, l_value, r_value),
        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && r_scalar.is_float() => fmul(module, l_type, res_id, l_value, r_value),

        (Vec(_, v_scalar), t_scalar) if t_scalar == *v_scalar && t_scalar.is_float() => vector_times_scalar(module, l_type, res_id, r_value, l_value),
        (t_scalar, Vec(_, v_scalar)) if t_scalar == *v_scalar && t_scalar.is_float() => vector_times_scalar(module, r_type, res_id, l_value, r_value),

        (Mat(_, m_scalar), t_scalar) if t_scalar == *m_scalar && t_scalar.is_float() => matrix_times_scalar(module, l_type, res_id, l_value, r_value),
        (t_scalar, Mat(_, m_scalar)) if t_scalar == *m_scalar && t_scalar.is_float() => matrix_times_scalar(module, r_type, res_id, r_value, l_value),

        (Vec(v_len, l_scalar), Mat(m_len, r_scalar)) if v_len == m_len && *l_scalar == *r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_type);

            module.instructions.push(Instruction::VectorTimesMatrix {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                vector: ValueId(l_value),
                matrix: ValueId(r_value),
            });
        },
        (Mat(m_len, l_scalar), Vec(v_len, r_scalar)) if v_len == m_len && *l_scalar == *r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_type);

            module.instructions.push(Instruction::MatrixTimesVector {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                matrix: ValueId(l_value),
                vector: ValueId(r_value),
            });
        },

        (Mat(l_len, l_scalar), Mat(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(l_type);

            module.instructions.push(Instruction::MatrixTimesMatrix {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                lhs: ValueId(l_value),
                rhs: ValueId(r_value),
            });
        },

        _ => return Err("Unsupported multiplication")
    }

    Ok(res_id)
}

pub fn dot(module: &mut Module, args: Vec<(TypeName, u32)>) -> Result<u32, &'static str> {
    use types::TypeName::*;

    if args.len() != 2 {
        return Err("Wrong number of arguments for Dot");
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    match (l_type, r_type) {
        (Vec(l_size, l_scalar), Vec(r_size, r_scalar)) if l_size == r_size && *l_scalar == *r_scalar && l_scalar.is_float() => {
            let res_type = module.register_type(*l_scalar);

            let result_id = module.get_id();

            module.instructions.push(Instruction::Dot {
                result_type: TypeId(res_type),
                result_id: ResultId(result_id),
                lhs: ValueId(l_value),
                rhs: ValueId(r_value),
            });

            Ok(result_id)
        },
        _ => Err("Invalid arguments for Dot")
    }
}

macro_rules! impl_math_op {
    ( $name:ident, $( $opcode:ident ),* ) => {
        pub fn $name(module: &mut Module, args: Vec<(TypeName, u32)>) -> Result<u32, &'static str> {
            use types::TypeName::*;

            if args.len() != 2 {
                return Err(concat!("Wrong number of arguments for ", stringify!($name)));
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
                        },
                        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && r_scalar.is_signed() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$sopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });
                        },

                        _ if l_type == r_type && r_type.is_integer() && !r_type.is_signed() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$uopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });
                        },
                        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && r_scalar.is_integer() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$uopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });
                        },

                        _ if l_type == r_type && r_type.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });
                        },
                        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && r_scalar.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });
                        },

                        _ => return Err(concat!("Unsupported ", stringify!($name), " operation"))
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
                        },
                        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && r_scalar.is_integer() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$iopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(l_value),
                                rhs: ValueId(r_value),
                            });
                        },

                        _ if l_type == r_type && r_type.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(r_value),
                                rhs: ValueId(l_value),
                            });
                        },
                        (Vec(l_len, l_scalar), Vec(r_len, r_scalar)) if l_len == r_len && *l_scalar == *r_scalar && l_scalar.is_float() => {
                            let res_type = module.register_type(l_type);

                            module.instructions.push(Instruction::$fopcode {
                                result_type: TypeId(res_type),
                                result_id: ResultId(result_id),
                                lhs: ValueId(r_value),
                                rhs: ValueId(l_value),
                            });
                        },

                        _ => return Err(concat!("Unsupported ", stringify!($name), " operation"))
                    }
                };
            }

            match_types!( $( $opcode ),* );

            Ok(result_id)
        }
    };
}

impl_math_op!(add, IAdd, FAdd);
impl_math_op!(substract, ISub, FSub);
impl_math_op!(divide, UDiv, SDiv, FDiv);
impl_math_op!(modulus, UMod, SMod, FMod);
