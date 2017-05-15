//! Exports Rust counterparts for all the common GLSL types, along with a few marker traits

use rasen::prelude::{Node, TypeName, TypedValue, NodeIndex};

use std::iter::Sum;
use std::marker::PhantomData;
use value::{GraphRef, Value, IntoValue};
use shader::{Shader, Input, Uniform, Output};
use std::ops::{Add, Sub, Mul, Div, Rem, Index};

pub trait Scalar: Copy + PartialOrd + PartialEq + IntoValue<Output=Self> + Into<Value<Self>> {
    fn zero() -> Self;
    fn one() -> Self;
}

pub trait Numerical : Scalar + Sum + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<Self, Output=Self> + Div<Self, Output=Self> + Rem<Self, Output=Self> {
    fn pow(x: Self, y: Self) -> Self;
}

pub trait Integer: Numerical {
    fn is_signed(&self) -> bool;
}

pub trait Floating : Numerical {
    fn is_double(&self) -> bool;
    fn sqrt(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
}

pub trait Vector<S>: Copy + From<Vec<S>> + Index<u32, Output=S> + IntoValue<Output=Self> + Into<Value<Self>> where S: Scalar {
    fn zero() -> Self;
    fn one() -> Self;
    fn component_count(&self) -> u32;
}

pub trait Matrix<V, S>: Into<Value<Self>> where V: Vector<S>, S: Scalar {
    fn identity() -> Self;
    fn column_count(&self) -> u32;
}

::rasen_codegen::decl_types!();
