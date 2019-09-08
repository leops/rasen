//! Exposes Rust counterparts of common GLSL functions

use std::{
    ops::{Add, Sub, Mul, Div, Index},
    cmp::PartialOrd,
};

use crate::{
    context::{Container, Context},
    types::*,
    value::{IntoValue, Value},
};

include! {
    concat!(env!("OUT_DIR"), "/operations.rs")
}

#[inline]
pub fn sample<C: Context>(
    sample: impl IntoValue<C, Output = Sampler>,
    index: impl IntoValue<C, Output = Vec2>,
) -> Value<C, Vec4> {
    <C as Container<Sampler>>::sample(sample.into_value(), index.into_value())
}
