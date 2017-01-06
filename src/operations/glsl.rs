use spirv_utils::instruction::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId,
};

use module::Module;
use glsl::GLSL::*;
use types::*;
use errors::*;

macro_rules! unary_vec {
    ( $name:ident, $op:ident ) => {
        #[inline]
        pub fn $name(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            if args.len() != 1 {
                Err(ErrorKind::WrongArgumentsCount(args.len(), 1))?;
            }

            let (arg_ty, arg_val) = args[0];
            let (res_type, scalar) = if let &Vec(_, scalar) = arg_ty {
                (module.register_type(scalar), scalar)
            } else {
                return Err(ErrorKind::BadArguments(Box::new([ arg_ty ])).into());
            };

            let res_id = module.get_id();
            let ext_id = module.import_set(String::from("GLSL.std.450"));

            module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                set: ValueId(ext_id),
                instruction: $op as u32,
                operands: Box::new([
                    Id(arg_val)
                ]),
            });

            Ok((scalar, res_id))
        }
    };
}

unary_vec!(sin, Sin);
unary_vec!(cos, Cos);
unary_vec!(tan, Tan);
unary_vec!(length, Length);

macro_rules! binary_any {
    ( $name:ident, $op:ident, $scode:ident, $ucode:ident, $fcode:ident ) => {
        #[inline]
        pub fn $name(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
            use types::TypeName::*;

            if args.len() != 2 {
                Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
            }

            let (l_type, l_value) = args[0];
            let (r_type, r_value) = args[1];

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

            let res_type = module.register_type(l_type);
            let res_id = module.get_id();

            let ext_id = module.import_set(String::from("GLSL.std.450"));

            module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                set: ValueId(ext_id),
                instruction: inst_id as u32,
                operands: Box::new([
                    Id(l_value), Id(r_value)
                ]),
            });

            Ok((l_type, res_id))
        }
    };
}

binary_any!(min, Min, SMin, UMin, FMin);
binary_any!(max, Max, SMax, UMax, FMax);

macro_rules! trinary_any {
    ($name:ident, $op:ident, $fcode:ident$(, $scode:ident, $ucode:ident )*) => {
        #[inline]
        pub fn $name(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
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

            let res_type = module.register_type(a_type);
            let res_id = module.get_id();

            let ext_id = module.import_set(String::from("GLSL.std.450"));

            module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                set: ValueId(ext_id),
                instruction: inst_id as u32,
                operands: Box::new([
                    Id(a_value), Id(b_value), Id(c_value)
                ]),
            });

            Ok((a_type, res_id))
        }
    };
}

trinary_any!(clamp, Clamp, FClamp, SClamp, UClamp);
trinary_any!(mix, Mix, FMix);

#[inline]
pub fn distance(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];

    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar => {
            let res_type = module.register_type(l_scalar);

            let res_id = module.get_id();
            let ext_id = module.import_set(String::from("GLSL.std.450"));

            module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(res_type),
                result_id: ResultId(res_id),
                set: ValueId(ext_id),
                instruction: Distance as u32,
                operands: Box::new([
                    Id(l_value), Id(r_value)
                ]),
            });

            Ok((l_scalar, res_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([
            l_type, r_type
        ])))?,
    }
}

#[inline]
pub fn reflect(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 2 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 2))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];

    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar => {
            let vec_type = module.register_type(l_type);

            let result_id = module.get_id();
            let ext_id = module.import_set(String::from("GLSL.std.450"));

            module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(vec_type),
                result_id: ResultId(result_id),
                set: ValueId(ext_id),
                instruction: Reflect as u32,
                operands: Box::new([
                    Id(l_value), Id(r_value)
                ]),
            });

            Ok((l_type, result_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([ l_type, r_type ])))?,
    }
}

#[inline]
pub fn refract(module: &mut Module, args: Vec<(&'static TypeName, u32)>) -> Result<(&'static TypeName, u32)> {
    use types::TypeName::*;

    if args.len() != 3 {
        Err(ErrorKind::WrongArgumentsCount(args.len(), 3))?;
    }

    let (l_type, l_value) = args[0];
    let (r_type, r_value) = args[1];
    let (i_type, i_value) = args[2];

    match (l_type, r_type) {
        (&Vec(l_size, l_scalar), &Vec(r_size, r_scalar)) if l_size == r_size && l_scalar == r_scalar && l_scalar == i_type && i_type.is_float() => {
            let vec_type = module.register_type(l_type);

            let res_id = module.get_id();
            let ext_id = module.import_set(String::from("GLSL.std.450"));

            module.instructions.push(Instruction::ExtInst {
                result_type: TypeId(vec_type),
                result_id: ResultId(res_id),
                set: ValueId(ext_id),
                instruction: Refract as u32,
                operands: Box::new([
                    Id(l_value), Id(r_value), Id(i_value)
                ]),
            });

            Ok((l_type, res_id))
        },
        _ => Err(ErrorKind::BadArguments(Box::new([ l_type, r_type, i_type ])))?,
    }
}
