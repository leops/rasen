use spirv_headers::*;
use rspirv::mr::{
    Instruction, Operand
};

use builder::Builder;
use types::*;
use errors::*;

#[inline]
fn imul<B: Builder>(builder: &mut B, res_type: &'static TypeName, res_id: u32, lhs: u32, rhs: u32) {
    let res_type = builder.register_type(res_type);

    builder.push_instruction(
        Instruction::new(
            Op::IMul,
            Some(res_type),
            Some(res_id),
            vec![
                Operand::IdRef(lhs),
                Operand::IdRef(rhs),
            ]
        )
    );
}
#[inline]
fn fmul<B: Builder>(builder: &mut B, res_type: &'static TypeName, res_id: u32, lhs: u32, rhs: u32) {
    let res_type = builder.register_type(res_type);

    builder.push_instruction(
        Instruction::new(
            Op::FMul,
            Some(res_type),
            Some(res_id), vec![
                Operand::IdRef(lhs),
                Operand::IdRef(rhs),
            ]
        )
    );
}
#[inline]
fn vector_times_scalar<B: Builder>(builder: &mut B, res_type: &'static TypeName, res_id: u32, vector: u32, scalar: u32) {
    let res_type = builder.register_type(res_type);

    builder.push_instruction(
        Instruction::new(
            Op::VectorTimesScalar,
            Some(res_type),
            Some(res_id),
            vec![
                Operand::IdRef(vector),
                Operand::IdRef(scalar),
            ]
        )
    );
}
#[inline]
fn matrix_times_scalar<B: Builder>(builder: &mut B, res_type: &'static TypeName, res_id: u32, matrix: u32, scalar: u32) {
    let res_type = builder.register_type(res_type);

    builder.push_instruction(
        Instruction::new(
            Op::MatrixTimesScalar,
            Some(res_type),
            Some(res_id),
            vec![
                Operand::IdRef(matrix),
                Operand::IdRef(scalar),
            ]
        )
    );
}

#[inline]
pub fn multiply<B: Builder>(builder: &mut B, args: &[(&'static TypeName, u32)]) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    let (l_arg, r_arg) = match args.len() {
        2 => (
            args[0],
            args[1],
        ),
        n if n > 2 => (
            multiply(builder, &args[0..n - 1])?,
            args[n - 1],
        ),
        n => bail!(ErrorKind::WrongArgumentsCount(n, 2)),
    };

    let (l_type, l_value) = l_arg;
    let (r_type, r_value) = r_arg;
    let res_id = builder.get_id();

    let res_type = match (l_type, r_type) {
        _ if l_type == r_type && r_type.is_integer() => {
            imul(builder, l_type, res_id, l_value, r_value);
            l_type
        },
        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => {
            imul(builder, l_type, res_id, l_value, r_value);
            l_type
        },

        _ if l_type == r_type && l_type.is_float() => {
            fmul(builder, l_type, res_id, l_value, r_value);
            l_type
        },
        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_float() => {
            fmul(builder, l_type, res_id, l_value, r_value);
            l_type
        },

        (&Vec(_, v_scalar), t_scalar) if t_scalar == v_scalar && t_scalar.is_float() => {
            vector_times_scalar(builder, l_type, res_id, r_value, l_value);
            l_type
        },
        (t_scalar, &Vec(_, v_scalar)) if t_scalar == v_scalar && t_scalar.is_float() => {
            vector_times_scalar(builder, r_type, res_id, l_value, r_value);
            r_type
        },

        (&Mat(_, &Vec(_, m_scalar)), t_scalar) if t_scalar == m_scalar && t_scalar.is_float() => {
            matrix_times_scalar(builder, l_type, res_id, l_value, r_value);
            l_type
        },
        (t_scalar, &Mat(_, &Vec(_, m_scalar))) if t_scalar == m_scalar && t_scalar.is_float() => {
            matrix_times_scalar(builder, r_type, res_id, r_value, l_value);
            r_type
        },

        (&Vec(v_len, l_scalar), &Mat(m_len, &Vec(_, r_scalar))) if v_len == m_len && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = builder.register_type(l_type);

            builder.push_instruction(
                Instruction::new(
                    Op::VectorTimesMatrix,
                    Some(res_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(l_value),
                        Operand::IdRef(r_value),
                     ]
                 )
             );

            l_type
        },
        (&Mat(m_len, &Vec(_, l_scalar)), &Vec(v_len, r_scalar)) if v_len == m_len && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = builder.register_type(r_type);

            builder.push_instruction(
                Instruction::new(
                    Op::MatrixTimesVector,
                    Some(res_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(l_value),
                        Operand::IdRef(r_value),
                    ]
                )
            );

            r_type
        },

        (&Mat(l_len, &Vec(_, l_scalar)), &Mat(r_len, &Vec(_, r_scalar))) if l_len == r_len && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = builder.register_type(l_type);

            builder.push_instruction(Instruction::new(Op::MatrixTimesMatrix, Some(res_type), Some(res_id), vec![ Operand::IdRef(l_value),
                Operand::IdRef(r_value),
             ]));

            l_type
        },

        _ => bail!(ErrorKind::BadArguments(Box::new([ l_type, r_type ]))),
    };

    Ok((res_type, res_id))
}

#[inline]
pub fn dot<B: Builder>(builder: &mut B, args: &[(&'static TypeName, u32)]) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        bail!(ErrorKind::WrongArgumentsCount(args.len(), 2));
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar && l_scalar.is_float() => {
            let res_type = builder.register_type(l_scalar);

            let result_id = builder.get_id();

            builder.push_instruction(Instruction::new(Op::Dot, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                Operand::IdRef(r_value),
             ]));

            Ok((l_scalar, result_id))
        },
        _ => bail!(ErrorKind::BadArguments(Box::new([ l_type, r_type ]))),
    }
}

macro_rules! impl_math_op {
    ( $name:ident, $node:ident, $variadic:expr, $( $opcode:ident ),* ) => {
        #[inline]
        pub fn $name<B: Builder>(builder: &mut B, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            let (l_arg, r_arg) = match args.len() {
                2 => (
                    args[0],
                    args[1],
                ),
                n if $variadic && n > 2 => (
                    $name(builder, args[0..n - 1].to_vec())?,
                    args[n - 1],
                ),
                n => bail!(ErrorKind::WrongArgumentsCount(n, 2)),
            };

            let result_id = builder.get_id();

            let (l_type, l_value) = l_arg;
            let (r_type, r_value) = r_arg;

            macro_rules! match_types {
                ( $uopcode:ident, $sopcode:ident, $fopcode:ident ) => {
                    match (l_type, r_type) {
                        _ if l_type == r_type && r_type.is_signed() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$sopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_signed() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$sopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },

                        _ if l_type == r_type && r_type.is_integer() && !r_type.is_signed() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$uopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$uopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },

                        _ if l_type == r_type && r_type.is_float() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$fopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_float() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(
                                Instruction::new(
                                    Op::$fopcode,
                                    Some(res_type),
                                    Some(result_id),
                                    vec![
                                        Operand::IdRef(l_value),
                                        Operand::IdRef(r_value),
                                    ]
                                )
                            );

                            l_type
                        },

                        _ => bail!(ErrorKind::BadArguments(Box::new([ l_type, r_type ]))),
                    }
                };
                ( $iopcode:ident, $fopcode:ident ) => {
                    match (l_type, r_type) {
                        _ if l_type == r_type && r_type.is_integer() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$iopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$iopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(l_value),
                                Operand::IdRef(r_value),
                             ]));

                            l_type
                        },

                        _ if l_type == r_type && r_type.is_float() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$fopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(r_value),
                                Operand::IdRef(l_value),
                             ]));

                            l_type
                        },
                        (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && l_scalar.is_float() => {
                            let res_type = builder.register_type(l_type);

                            builder.push_instruction(Instruction::new(Op::$fopcode, Some(res_type), Some(result_id), vec![ Operand::IdRef(r_value),
                                Operand::IdRef(l_value),
                             ]));

                            l_type
                        },

                        _ => bail!(ErrorKind::BadArguments(Box::new([ l_type, r_type ]))),
                    }
                };
            }

            let res_type = match_types!( $( $opcode ),* );
            Ok((res_type, result_id))
        }
    };
}

impl_math_op!(add, Add, true, IAdd, FAdd);
impl_math_op!(subtract, Subtract, true, ISub, FSub);
impl_math_op!(divide, Divide, true, UDiv, SDiv, FDiv);
impl_math_op!(modulus, Modulus, false, UMod, SMod, FMod);
