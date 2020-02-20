use std::ops::{Add, Div, Index, Mul, Rem, Sub};

use rasen::prelude::{Node, NodeIndex, TypeName, TypedValue};

use crate::{
    context::{Container, Context},
    module::with_graph,
    types::*,
    value::Value,
};

pub(crate) type ParseNode = NodeIndex;

pub enum Parse {}

include! {
    concat!(env!("OUT_DIR"), "/parse.rs")
}

impl Context for Parse {}
