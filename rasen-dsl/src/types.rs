//! Exports Rust counterparts for all the common GLSL types, along with a few marker traits

use rasen::prelude::{Node, TypeName, TypedValue, NodeIndex};

use std::iter::Sum;
use std::marker::PhantomData;
use value::{GraphRef, Value, IntoValue};
use shader::{Shader, Input, Uniform, Output};
use std::ops::{Add, Sub, Mul, Div, Rem, Index};

pub trait ValueIter<T> {
    type Iter: Iterator<Item=Value<T>>;
    fn iter<'a>(obj: &Self) -> Self::Iter;
}

pub trait Scalar: 'static + Copy + IntoValue<Output=Self> + Into<Value<Self>> + ValueIter<Self> + PartialOrd + PartialEq {
    fn zero() -> Self;
    fn one() -> Self;
}

pub trait Numerical : Scalar + Sum + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<Self, Output=Self> + Div<Self, Output=Self> + Rem<Self, Output=Self> {
    fn pow(x: Self, y: Self) -> Self;
}

pub trait Integer: Numerical {
    fn is_signed() -> bool;
}

pub trait Floating : Numerical {
    fn is_double() -> bool;
    fn sqrt(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
}

pub trait Vector<S>: Copy + IntoValue<Output=Self> + Into<Value<Self>> + ValueIter<S> + From<Vec<S>> + Index<u32, Output=S> where S: Scalar {
    fn zero() -> Self;
    fn one() -> Self;
    fn component_count() -> u32;
}

pub trait Matrix<V, S>: Copy + IntoValue<Output=Self> + Into<Value<Self>> + ValueIter<S> + Index<u32, Output=S> where V: Vector<S>, S: Scalar {
    fn identity() -> Self;
    fn column_count() -> u32;
}

::rasen_codegen::decl_types!();
