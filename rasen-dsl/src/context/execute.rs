use std::ops::{Add, Sub, Mul, Div, Rem, Index};

use crate::{context::{Container, Context}, value::Value, types::*};

pub enum Execute {}

include! {
    concat!(env!("OUT_DIR"), "/execute.rs")
}

impl Context for Execute {}
