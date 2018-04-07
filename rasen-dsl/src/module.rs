//! Module builder utility

use std::rc::Rc;
use std::cell::{RefCell, RefMut};
#[cfg(feature = "functions")]
use std::ops::FnOnce;

use rasen::prelude::{
    Module as CoreModule, ShaderType,
    build_program, build_program_assembly,
};
#[cfg(feature = "functions")]
use rasen::prelude::Node;

use rasen::module::FunctionRef;
use rasen::errors;

use value::Value;
#[cfg(feature = "functions")]
use value::IntoValue;

pub(crate) type ModuleRef<'a> = RefMut<'a, CoreModule>;

/// The Module builder, a lightweight wrapper around a shared mutable Graph
#[derive(Clone, Debug, Default)]
pub struct Module {
    module: Rc<RefCell<CoreModule>>,
}

impl Module {
    pub fn new() -> Module {
        Default::default()
    }

    pub(crate) fn borrow_mut<'a>(&'a self) -> ModuleRef<'a> {
        self.module.borrow_mut()
    }

    pub fn function<F>(&self, thunk: F) -> Function<F> {
        Function::new(self.clone(), thunk)
    }

    pub fn build(&self, ty: ShaderType) -> errors::Result<Vec<u8>> {
        build_program(&self.module.borrow() as &CoreModule, ty)
    }

    pub fn build_assembly(&self, ty: ShaderType) -> errors::Result<String> {
        build_program_assembly(&self.module.borrow() as &CoreModule, ty)
    }
}

/// Shader attribute
pub trait Input<T> {
    fn input(&self, location: u32) -> Value<T>;
}

/// Shader uniform
pub trait Uniform<T> {
    fn uniform(&self, location: u32) -> Value<T>;
}

/// Shader outputs
pub trait Output<T> {
    fn output(&self, location: u32, value: Value<T>);
}

#[derive(Clone)]
pub struct Function<F> {
    pub(crate) module: Module,
    pub(crate) func: FunctionRef,
    pub(crate) thunk: F,
}

impl<F> Function<F> {
    pub fn new(module: Module, thunk: F) -> Function<F> {
        let func = module.module.borrow_mut().add_function();
        Function {
            func, thunk,
            module,
        }
    }

    #[cfg(feature = "functions")]
    pub(crate) fn ret_impl<A, S, R>(module: &Module, func: FunctionRef, source: S) where F: FnOnce<A, Output = Value<R>>, S: IntoValue<Output = R>, Value<R>: IntoValue {
        let src = match source.into_value() {
            Value::Abstract { index, .. } => index,
            source @ Value::Concrete(_) => {
                let module = module.module.borrow_mut();
                let mut graph = RefMut::map(
                    module,
                    |module| &mut module[func],
                );
                source.get_index(graph)
            },
        };

        let mut module = module.module.borrow_mut();
        let graph = &mut module[func];
        let sink = graph.add_node(Node::Return);
        graph.add_edge(src, sink, 0);
    }

    #[cfg(feature = "functions")]
    pub fn ret<A, S, R>(&self, source: S) where F: FnOnce<A, Output = Value<R>>, S: IntoValue<Output = R>, Value<R>: IntoValue {
        Self::ret_impl(&self.module, self.func, source);
    }
}

/// Function input
pub trait Parameter<T> {
    fn parameter(&self, location: u32) -> Value<T>;
}
