use std::ops::{Add, Sub, Mul, Div, Rem, Index};

use crate::{value::Value, types::*};

pub mod parse;
pub mod execute;

include! {
    concat!(env!("OUT_DIR"), "/container.rs")
}

include! {
    concat!(env!("OUT_DIR"), "/context.rs")
}
