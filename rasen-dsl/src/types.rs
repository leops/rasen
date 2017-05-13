use rasen::prelude::{Node, TypeName, TypedValue, NodeIndex};

use std::ops::Index;
use std::marker::PhantomData;
use shader::{Shader, Input, Uniform, Output};
use operations::{GraphRef, Value, IntoValue};

pub trait Scalar: Copy + Clone {
    // Marker
}

pub trait Numerical : Scalar {
    // Marker
}

pub trait Integer: Numerical {
    fn is_signed() -> bool;
}

pub trait Floating : Numerical {
    fn is_double() -> bool;
}

pub trait Vector<S>: Index<u32> where S: Scalar {
    fn component_count() -> u32;
}

pub trait Matrix<V, S> where V: Vector<S>, S: Scalar {
    fn column_count() -> u32;
}

::rasen_codegen::decl_types!();
