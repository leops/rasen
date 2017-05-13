use rasen::prelude::{ShaderType, build_program};
use rasen::errors;

use operations::{GraphRef, Value};

/// Shader builder
pub struct Shader {
    pub graph: GraphRef,
}

impl Shader {
    pub fn new() -> Shader {
        Shader {
            graph: Default::default(),
        }
    }

    pub fn build(&self, ty: ShaderType) -> errors::Result<Vec<u8>> {
        build_program(&self.graph.borrow(), ty)
    }
}

/// Shader attribute
pub trait Input<T> {
    fn input(&self, location: u32) -> Value<T>;
}

/// Shader uniform
pub trait Uniform<T> {
    fn uniform(&self, location: u32) -> Value<T>;
}

/// Shader outputs
pub trait Output<T> {
    fn output(&self, location: u32, value: Value<T>);
}
