//! Definition and implentations of all the graph operations

use spirv_utils::*;
use spirv_utils::desc::{
    Id, ResultId, TypeId, ValueId,
};
use spirv_utils::instruction::*;

use super::Module;
use super::types::*;
use operations;

/// All the supported operations
#[derive(Debug, Copy, Clone)]
pub enum Node {
    /// Create an input with a location and a type
    ///
    /// Incoming values from other nodes are ignored
    Input(u32, &'static TypeName),

    /// Create an output with a location and a type
    ///
    /// Doesn't need to be an output of the graph, but all the outputs should use this type
    Output(u32, &'static TypeName),

    /// Declare a new constant
    ///
    /// Incoming values from other nodes are ignored
    Constant(TypedValue),

    /// Normalize a vector
    ///
    /// Takes a single parameter
    Normalize,

    /// Add some values
    ///
    /// For the moment, only 2 parameters are supported
    Add,

    /// Substract a value from another
    ///
    /// Takes 2 parameters
    Substract,

    /// Multiply some values
    ///
    /// For the moment, only 2 parameters are supported
    Multiply,

    /// Divide a value by another
    ///
    /// Takes 2 parameters
    Divide,

    /// Compute the modulus of a value by another
    ///
    /// Takes 2 parameters
    Modulus,

    /// Clamp a value in a range
    ///
    /// Takes 3 parameters: the value to be clamped, the minimum, and the maximum
    Clamp,

    /// Compute the dot product of 2 vectors
    ///
    /// Takes 2 parameters
    Dot,

    /// Compute the cross product of 2 vectors
    ///
    /// Takes 2 parameters
    Cross,

    /// Round a number to the largest lower or equal integer
    ///
    /// Takes a single parameter
    Floor,

    /// Round a number to the nearest integer
    ///
    /// Takes a single parameter
    Ceil,

    /// Round a number to the smallest higher or equal integer
    ///
    /// Takes a single parameter
    Round,

    /// Compute the sinus of an angle in radians
    ///
    /// Takes a single parameter
    Sin,

    /// Compute the cosinus of an angle in radians
    ///
    /// Takes a single parameter
    Cos,

    /// Compute the tangent of an angle in radians
    ///
    /// Takes a single parameter
    Tan,

    /// Raise a number to a power
    ///
    /// Takes 2 parameters
    Pow,

    /// Returns the smallest value of all its arguments
    ///
    /// For the moment, only 2 parameters are supported
    Min,

    /// Return the greatest value of all its arguments
    ///
    /// For the moment, only 2 parameters are supported
    Max,

    /// Computes the length of a vector
    ///
    /// Takes a single parameter
    Length,

    /// Computes the distance between 2 points
    ///
    /// Takes 2 parameters
    Distance,

    /// Reflect a vector against a surface normal
    ///
    /// Takes 2 parameters
    Reflect,

    /// Computes the refraction of a vector using a surface normal and a refraction indice
    ///
    /// Takes 3 parameters
    Refract,

    /// Computes a linear interpolation between two values
    ///
    /// Takes 3 parameters
    Mix,
}

impl Node {
    /// Insert this Node into a Program
    pub fn get_result(&self, module: &mut Module, args: Vec<(TypeName, u32)>) -> Result<u32, &'static str> {
        use glsl::GLSL::*;

        macro_rules! impl_glsl_call {
            ( $function:expr, $argc:expr ) => {
                {
                    if args.len() != $argc {
                        return Err(concat!("Wrong number of arguments for ", stringify!($function)));
                    }

                    let ext_id = module.import_set(String::from("GLSL.std.450"));

                    let res_type = module.register_type(args[0].0);

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

                    Ok(result_id)
                }
            };
        }

        match *self {
            Node::Output(location, attr_type) => {
                if args.len() != 1 {
                    return Err("Wrong number of arguments for Output");
                }

                let type_id = module.register_type(*attr_type);

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
                    obj: ValueId(args[0].1),
                    memory_access: desc::MemoryAccess::empty()
                });

                Ok(var_id)
            },

            Node::Input(location, attr_type) => {
                let type_id = module.register_type(*attr_type);

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

                Ok(res_id)
            },

            Node::Constant(ref const_type) => module.register_constant(*const_type),

            Node::Add => operations::add(module, args),
            Node::Substract => operations::substract(module, args),
            Node::Multiply => operations::multiply(module, args),
            Node::Divide => operations::divide(module, args),
            Node::Modulus => operations::modulus(module, args),
            Node::Dot => operations::dot(module, args),

            Node::Clamp => operations::clamp(module, args),
            Node::Mix => operations::mix(module, args),

            Node::Normalize => impl_glsl_call!(Normalize, 1),
            Node::Cross => impl_glsl_call!(Cross, 2),

            Node::Pow => impl_glsl_call!(Pow, 2),
            Node::Floor => impl_glsl_call!(Floor, 1),
            Node::Ceil => impl_glsl_call!(Ceil, 1),
            Node::Round => impl_glsl_call!(Round, 1),

            Node::Sin => operations::sin(module, args),
            Node::Cos => operations::cos(module, args),
            Node::Tan => operations::tan(module, args),

            Node::Min => operations::min(module, args),
            Node::Max => operations::max(module, args),

            Node::Length => operations::length(module, args),
            Node::Distance => operations::distance(module, args),
            Node::Reflect => operations::reflect(module, args),
            Node::Refract => operations::refract(module, args),
        }
    }
}
