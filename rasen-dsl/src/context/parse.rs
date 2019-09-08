use std::ops::{Add, Sub, Mul, Div, Rem, Index};

use rasen::prelude::{NodeIndex, Node, TypeName, TypedValue};

use crate::{context::{Container, Context}, value::Value, types::*, module::with_graph};

pub(crate) type ParseNode = NodeIndex;

pub enum Parse {}

include! {
    concat!(env!("OUT_DIR"), "/parse.rs")
}

impl Context for Parse {}
