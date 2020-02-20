//! Definitions for the Value type

use std::ops::{Add, Div, Mul, Rem, Sub};

use crate::context::{execute::Execute, Container};

pub struct Value<C: Container<T> + ?Sized, T>(pub(crate) C::Value);

impl<C: Container<T>, T> Copy for Value<C, T> where C::Value: Copy {}

impl<C: Container<T>, T> Clone for Value<C, T>
where
    C::Value: Clone,
{
    fn clone(&self) -> Self {
        Value(self.0.clone())
    }
}

impl<T: Copy> Value<Execute, T>
where
    Execute: Container<T, Value = T>,
{
    pub fn of(value: T) -> Self {
        Value(value)
    }

    pub fn read(self) -> T {
        self.0
    }
}

impl<C, T, R> Add<Value<C, R>> for Value<C, T>
where
    T: Add<R>,
    R: Copy,
    T::Output: Copy,
    C: Container<T> + Container<R> + Container<T::Output>,
{
    type Output = Value<C, T::Output>;
    fn add(self, rhs: Value<C, R>) -> Self::Output {
        C::add(self, rhs)
    }
}

impl<C, T, R> Sub<Value<C, R>> for Value<C, T>
where
    T: Sub<R>,
    R: Copy,
    T::Output: Copy,
    C: Container<T> + Container<R> + Container<T::Output>,
{
    type Output = Value<C, T::Output>;
    fn sub(self, rhs: Value<C, R>) -> Self::Output {
        C::sub(self, rhs)
    }
}

impl<C, T, R> Mul<Value<C, R>> for Value<C, T>
where
    T: Mul<R>,
    R: Copy,
    T::Output: Copy,
    C: Container<T> + Container<R> + Container<T::Output>,
{
    type Output = Value<C, T::Output>;
    fn mul(self, rhs: Value<C, R>) -> Self::Output {
        C::mul(self, rhs)
    }
}

impl<C, T, R> Div<Value<C, R>> for Value<C, T>
where
    T: Div<R>,
    R: Copy,
    T::Output: Copy,
    C: Container<T> + Container<R> + Container<T::Output>,
{
    type Output = Value<C, T::Output>;
    fn div(self, rhs: Value<C, R>) -> Self::Output {
        C::div(self, rhs)
    }
}

impl<C, T, R> Rem<Value<C, R>> for Value<C, T>
where
    T: Rem<R>,
    R: Copy,
    T::Output: Copy,
    C: Container<T> + Container<R> + Container<T::Output>,
{
    type Output = Value<C, T::Output>;
    fn rem(self, rhs: Value<C, R>) -> Self::Output {
        C::rem(self, rhs)
    }
}

pub trait IntoValue<C> {
    type Output;
    fn into_value(self) -> Value<C, Self::Output>
    where
        C: Container<Self::Output>;
}

impl<C: Container<T>, T> IntoValue<C> for Value<C, T> {
    type Output = T;

    fn into_value(self) -> Value<C, T>
    where
        C: Container<T>,
    {
        self
    }
}
