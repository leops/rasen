use std::ops::{Add, Div, Index, Mul, Rem, Sub};

use crate::{types::*, value::Value};

pub mod execute;
pub mod parse;

include! {
    concat!(env!("OUT_DIR"), "/container.rs")
}

include! {
    concat!(env!("OUT_DIR"), "/context.rs")
}
