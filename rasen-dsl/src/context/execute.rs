use std::ops::{Add, Div, Index, Mul, Rem, Sub};

use crate::{
    context::{Container, Context},
    types::*,
    value::Value,
};

pub enum Execute {}

include! {
    concat!(env!("OUT_DIR"), "/execute.rs")
}

impl Context for Execute {}
