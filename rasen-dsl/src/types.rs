//! Exports Rust counterparts for all the common GLSL types, along with a few marker traits

use rasen::prelude::{Node, TypeName, TypedValue, NodeIndex};

use std::ops::Index;
use std::marker::PhantomData;
use value::{GraphRef, Value, FuncKind, IntoValue};
use module::{Module, Input, Uniform, Output, Function, Parameter};

pub mod traits {
    use std::iter::Sum;
    use value::{Value, IntoValue};
    use std::ops::{Add, Sub, Mul, Div, Rem, Index};

    pub trait ValueIter<T> {
        type Iter: Iterator<Item=Value<T>>;
        fn iter(obj: &Self) -> Self::Iter;
    }

    pub trait Scalar: 'static + Copy + IntoValue<Output=Self> + Into<Value<Self>> + ValueIter<Self> + PartialOrd + PartialEq {
        fn zero() -> Self;
        fn one() -> Self;
    }

    pub trait Numerical: Scalar + Sum + Add<Self, Output=Self> + Sub<Self, Output=Self> + Mul<Self, Output=Self> + Div<Self, Output=Self> + Rem<Self, Output=Self> {
        fn pow(x: Self, y: Self) -> Self;
    }

    pub trait Integer: Numerical {
        fn is_signed() -> bool;
    }

    pub trait Floating: Numerical {
        fn is_double() -> bool;
        fn sqrt(self) -> Self;
        fn floor(self) -> Self;
        fn ceil(self) -> Self;
        fn round(self) -> Self;
        fn sin(self) -> Self;
        fn cos(self) -> Self;
        fn tan(self) -> Self;
    }

    pub trait Vector<S>: Copy + IntoValue<Output=Self> + Into<Value<Self>> + ValueIter<S> + From<Vec<S>> + Index<u32, Output=S> where S: Scalar {
        fn zero() -> Self;
        fn one() -> Self;
        fn component_count() -> u32;
    }

    pub trait Matrix<V, S>: Copy + IntoValue<Output=Self> + Into<Value<Self>> + ValueIter<S> + Index<u32, Output=S> where V: Vector<S>, S: Scalar {
        fn identity() -> Self;
        fn column_count() -> u32;
    }
}

use self::traits::*;

#[derive(Copy, Clone)]
pub struct Sampler(pub Vec4);

impl IntoValue for Sampler {
    type Output = Self;

    fn into_value(self) -> Value<Self> {
        Value::Concrete(self)
    }

    /// Registers this value into a Graph and returns the node index
    fn get_index(&self, _graph: GraphRef) -> NodeIndex<u32> {
        unimplemented!()
    }
}

impl Uniform<Sampler> for Module {
    #[inline]
    fn uniform(&self, location: u32) -> Value<Sampler> {
        let index = {
            let mut module = self.borrow_mut();
            module.main.add_node(Node::Uniform(location, TypeName::SAMPLER2D))
        };

        Value::Abstract {
            module: self.clone(),
            function: FuncKind::Main,
            index,
            ty: PhantomData,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));
