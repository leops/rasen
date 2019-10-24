//! Exposes Rust counterparts of common GLSL functions

use std::{
    cmp::PartialOrd,
    ops::{Add, Div, Index, Mul, Sub},
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
pub fn sample<C, V>(
    sample: impl IntoValue<C, Output = Sampler<V>>,
    index: impl IntoValue<C, Output = Vec2>,
) -> Value<C, V>
where
    C: Context + Container<Sampler<V>> + Container<V>,
    V: Copy,
{
    <C as Container<Sampler<V>>>::sample(sample.into_value(), index.into_value())
}
