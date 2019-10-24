//! Exports Rust counterparts for all the common GLSL types, along with a few marker traits

use rasen::prelude::{Dim, TypeName};

use std::ops::{Add, Div, Index, Mul, Rem, Sub};

use crate::{
    context::{Container, Context},
    value::{IntoValue, Value},
};

pub trait AsTypeName {
    const TYPE_NAME: &'static TypeName;
}

pub trait GenType: Copy {
    fn zero() -> Self;
    fn one() -> Self;
    fn min(self, rhs: Self) -> Self;
    fn max(self, rhs: Self) -> Self;
}

pub trait Numerical: GenType {
    fn pow(self, rhs: Self) -> Self;
}

pub trait Floating: Numerical {
    fn sqrt(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn ln(self) -> Self;
    fn abs(self) -> Self;
}

pub trait Vector: GenType {
    type Scalar: Numerical;
    fn spread(v: Self::Scalar) -> Self;
}

pub trait VectorFloating: Vector
where
    Self::Scalar: Floating,
{
    fn dot(&self, rhs: &Self) -> Self::Scalar;

    fn normalize(&self) -> Self;
    fn length_squared(&self) -> Self::Scalar;

    fn length(&self) -> Self::Scalar {
        self.length_squared().sqrt()
    }
}

pub trait Vector3: Vector {
    fn cross(&self, rhs: &Self) -> Self;
}

pub trait Matrix {
    fn inverse(self) -> Self;
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));

#[derive(Copy, Clone, Debug)]
pub struct Sampler<V>(pub V);

impl<V: Vector> AsTypeName for Sampler<V>
where
    <V as Vector>::Scalar: AsTypeName,
{
    const TYPE_NAME: &'static TypeName =
        &TypeName::Sampler(<<V as Vector>::Scalar as AsTypeName>::TYPE_NAME, Dim::Dim2D);
}
