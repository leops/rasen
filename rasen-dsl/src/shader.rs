//! Shader builder utility

use rasen::prelude::{ShaderType, build_program, build_program_assembly};
use rasen::errors;

use value::{GraphRef, Value};

/// The Shader builder, a lightweight wrapper around a shared mutable Graph
#[derive(Default)]
pub struct Shader {
    pub graph: GraphRef,
}

impl Shader {
    pub fn new() -> Shader {
        Default::default()
    }

    pub fn build(&self, ty: ShaderType) -> errors::Result<Vec<u8>> {
        build_program(&self.graph.borrow(), ty)
    }

    pub fn build_assembly(&self, ty: ShaderType) -> errors::Result<String> {
        build_program_assembly(&self.graph.borrow(), ty)
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
