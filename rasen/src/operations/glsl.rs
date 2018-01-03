use spirv_headers::*;
use spirv_headers::GLOp::*;
use rspirv::mr::{
    Instruction, Operand
};

use builder::Builder;
use types::*;
use errors::*;

macro_rules! unary_vec {
    ( $name:ident, $op:ident ) => {
        #[inline]
        pub fn $name(builder: &mut Builder, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            if args.len() != 1 {
                Err(ErrorKind::WrongArgumentsCount(args.len(), 1))?;
            }

            let (arg_ty, arg_val) = args[0];
            let (res_type, scalar) = if let Vec(_, scalar) = *arg_ty {
                (builder.register_type(scalar), scalar)
            } else {
                return Err(ErrorKind::BadArguments(Box::new([ arg_ty ])).into());
            };

            let res_id = builder.get_id();
            let ext_id = builder.import_set("GLSL.std.450");

            builder.push_instruction(
                Instruction::new(
                    Op::ExtInst,
                    Some(res_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(ext_id),
                        Operand::LiteralExtInstInteger($op as Word),
                        Operand::IdRef(arg_val)
                    ]
                )
            );

            Ok((scalar, res_id))
        }
    };
}

unary_vec!(sin, Sin);
unary_vec!(cos, Cos);
unary_vec!(tan, Tan);
unary_vec!(length, Length);

macro_rules! variadic_any {
    ( $name:ident, $op:ident, $scode:ident, $ucode:ident, $fcode:ident ) => {
        #[inline]
        pub fn $name(builder: &mut Builder, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            let (l_arg, r_arg) = match args.len() {
                2 => (
                    args[0],
                    args[1],
                ),
                n if n > 2 => (
                    $name(builder, args[0..n - 1].to_vec())?,
                    args[n - 1],
                ),
                n => Err(ErrorKind::WrongArgumentsCount(n, 2))?,
            };

            let (l_type, l_value) = l_arg;
            let (r_type, r_value) = r_arg;

            let inst_id = match (l_type, r_type) {
                _ if l_type == r_type && r_type.is_signed() => $scode,
                (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_signed() => $scode,

                _ if l_type == r_type && r_type.is_integer() && !r_type.is_signed() => $ucode,
                (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_integer() => $ucode,

                _ if l_type == r_type && r_type.is_float() => $fcode,
                (&Vec(l_len, l_scalar), &Vec(r_len, r_scalar)) if l_len == r_len && l_scalar == r_scalar && r_scalar.is_float() => $fcode,

                _ => Err(ErrorKind::BadArguments(Box::new([
                    l_type, r_type
                ])))?,
            };

            let res_type = builder.register_type(l_type);
            let res_id = builder.get_id();

            let ext_id = builder.import_set("GLSL.std.450");

            builder.push_instruction(
                Instruction::new(
                    Op::ExtInst,
                    Some(res_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(ext_id),
                        Operand::LiteralExtInstInteger(inst_id as Word),
                        Operand::IdRef(l_value),
                        Operand::IdRef(r_value)
                    ]
                )
            );

            Ok((l_type, res_id))
        }
    };
}

variadic_any!(min, Min, SMin, UMin, FMin);
variadic_any!(max, Max, SMax, UMax, FMax);

macro_rules! trinary_any {
    ($name:ident, $op:ident, $fcode:ident$(, $scode:ident, $ucode:ident )*) => {
        #[inline]
        pub fn $name(builder: &mut Builder, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            if args.len() != 3 {
                Err(ErrorKind::WrongArgumentsCount(args.len(), 3))?;
            }

            let (a_type, a_value) = args[0];
            let (b_type, b_value) = args[1];
            let (c_type, c_value) = args[2];

            let inst_id = match (a_type, b_type, c_type) {
                $(
                    _ if a_type == b_type && b_type == c_type && a_type.is_signed() => $scode,
                    (&Vec(a_len, a_scalar), &Vec(b_len, b_scalar), &Vec(c_len, c_scalar)) if a_len == b_len && b_len == c_len && a_scalar == b_scalar && b_scalar == c_scalar && a_scalar.is_signed() => $scode,

                    _ if a_type == b_type && b_type == c_type && a_type.is_integer() && !a_type.is_signed() => $ucode,
                    (&Vec(a_len, a_scalar), &Vec(b_len, b_scalar), &Vec(c_len, c_scalar)) if a_len == b_len && b_len == c_len && a_scalar == b_scalar && b_scalar == c_scalar && a_scalar.is_integer() && !a_type.is_signed() => $ucode,
                )*

                _ if a_type == b_type && b_type == c_type && a_type.is_float() => $fcode,
                (&Vec(a_len, a_scalar), &Vec(b_len, b_scalar), &Vec(c_len, c_scalar)) if a_len == b_len && b_len == c_len && a_scalar == b_scalar && b_scalar == c_scalar && a_scalar.is_float() => $fcode,

                _ => Err(ErrorKind::BadArguments(Box::new([
                    a_type, b_type, c_type
                ])))?,
            };

            let res_type = builder.register_type(a_type);
            let res_id = builder.get_id();

            let ext_id = builder.import_set("GLSL.std.450");

            builder.push_instruction(
                Instruction::new(
                    Op::ExtInst,
                    Some(res_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(ext_id),
                        Operand::LiteralExtInstInteger(inst_id as Word),
                        Operand::IdRef(a_value),
                        Operand::IdRef(b_value),
                        Operand::IdRef(c_value)
                    ]
                )
            );

            Ok((a_type, res_id))
        }
    };
}

trinary_any!(clamp, Clamp, FClamp, SClamp, UClamp);
trinary_any!(mix, Mix, FMix);

#[inline]
pub fn distance(builder: &mut Builder, args: &[(&'static TypeName, u32)]) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];

    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar => {
            let res_type = builder.register_type(l_scalar);

            let res_id = builder.get_id();
            let ext_id = builder.import_set("GLSL.std.450");

            builder.push_instruction(
                Instruction::new(
                    Op::ExtInst,
                    Some(res_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(ext_id),
                        Operand::LiteralExtInstInteger(Distance as u32),
                        Operand::IdRef(l_value),
                        Operand::IdRef(r_value)
                    ]
                )
            );

            Ok((l_scalar, res_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([
            l_type, r_type
        ])))?,
    }
}

#[inline]
pub fn reflect(builder: &mut Builder, args: &[(&'static TypeName, u32)]) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];

    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar => {
            let vec_type = builder.register_type(l_type);

            let result_id = builder.get_id();
            let ext_id = builder.import_set("GLSL.std.450");

            builder.push_instruction(
                Instruction::new(
                    Op::ExtInst,
                    Some(vec_type),
                    Some(result_id),
                    vec![
                        Operand::IdRef(ext_id),
                        Operand::LiteralExtInstInteger(Reflect as u32),
                        Operand::IdRef(l_value),
                        Operand::IdRef(r_value)
                    ]
                )
            );

            Ok((l_type, result_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([ l_type, r_type ])))?,
    }
}

#[inline]
pub fn refract(builder: &mut Builder, args: &[(&'static TypeName, u32)]) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 3 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 3))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    let (i_type, i_value) = args[2];

    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar && l_scalar == i_type && i_type.is_float() => {
            let vec_type = builder.register_type(l_type);

            let res_id = builder.get_id();
            let ext_id = builder.import_set("GLSL.std.450");

            builder.push_instruction(
                Instruction::new(
                    Op::ExtInst,
                    Some(vec_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(ext_id),
                        Operand::LiteralExtInstInteger(Refract as u32),
                        Operand::IdRef(l_value),
                        Operand::IdRef(r_value),
                        Operand::IdRef(i_value)
                    ]
                )
            );

            Ok((l_type, res_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([ l_type, r_type, i_type ])))?,
    }
}

#[inline]
pub fn sample(builder: &mut Builder, args: &[(&'static TypeName, u32)]) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (image_type, image_value) = args[0];
    let (coords_type, coords_value) = args[1];

    match (image_type, coords_type) {
        (&Sampler(sampled_type, Dim::Dim1D), &Vec(1, coords_scalar)) |
        (&Sampler(sampled_type, Dim::Dim2D), &Vec(2, coords_scalar)) |
        (&Sampler(sampled_type, Dim::Dim3D), &Vec(3, coords_scalar)) |
        (&Sampler(sampled_type, Dim::DimCube), &Vec(3, coords_scalar)) |
        (&Sampler(sampled_type, Dim::DimRect), &Vec(2, coords_scalar)) |
        (&Sampler(sampled_type, Dim::DimBuffer), &Vec(1, coords_scalar)) |

        (&Sampler(sampled_type, Dim::DimBuffer), coords_scalar) |
        (&Sampler(sampled_type, Dim::Dim1D), coords_scalar) if sampled_type.is_num() && coords_scalar.is_float() => {
            let res_type = match *sampled_type {
                Int(true) => TypeName::IVEC4,
                Int(false) => TypeName::UVEC4,
                Float(false) => TypeName::VEC4,
                Float(true) => TypeName::DVEC4,
                _ => unreachable!(),
            };

            let vec_type = builder.register_type(res_type);
            let res_id = builder.get_id();

            builder.push_instruction(
                Instruction::new(
                    Op::ImageSampleImplicitLod,
                    Some(vec_type),
                    Some(res_id),
                    vec![
                        Operand::IdRef(image_value),
                        Operand::IdRef(coords_value)
                    ]
                )
            );

            Ok((res_type, res_id))
        },
        
        _ => Err(ErrorKind::BadArguments(Box::new([ image_type, coords_type ])))?,
    }
}
