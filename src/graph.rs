//! Enumerations used in a shader graph

/// Define the type of a shader value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TypeName {
    Float,
    Vec2,
    Vec3,
    Vec4
}

/// A typed shader value (used for constants)
#[derive(Debug, Copy, Clone)]
pub enum TypedValue {
    Float(f32),
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
    Vec4(f32, f32, f32, f32),
}

/// All the supported operations
#[derive(Debug, Copy, Clone)]
pub enum Node {
    /// Create an input with a location and a type
    ///
    /// Incoming values from other nodes are ignored
    Input(u32, TypeName),

    /// Create an output with a location and a type
    ///
    /// Doesn't need to be an output of the graph, but all the outputs should use this type
    Output(u32, TypeName),

    /// Declare a new constant
    ///
    /// Incoming values from other nodes are ignored
    Constant(TypedValue),

    /// Normalize a vector
    ///
    /// Takes a single parameter
    Normalize,

    /// Multiply some values
    ///
    /// Form the moment, only 2 parameters are supported
    Multiply,

    /// Clamp a value in a range
    ///
    /// Takes 3 parameters: the value to be clamped, the minimum, and the maximum
    Clamp,

    /// Compute the dot product of 2 vectors
    ///
    /// Takes 2 parameters
    Dot
}
