//! Module builder utility

use std::{
    cell::{RefCell, RefMut},
    convert::TryFrom,
    ops::FnOnce,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

use rasen::prelude::{
    build_program, build_program_assembly, BuiltIn, Module as CoreModule, ModuleBuilder, Node,
    VariableName,
};

use rasen::{errors, module::FunctionRef};

use value::{IntoValue, Value};

pub(crate) type ModuleRef<'a> = RefMut<'a, CoreModule>;

/// The Module builder, a lightweight wrapper around a shared mutable Graph
#[derive(Clone, Debug, Default)]
pub struct Module {
    module: Rc<RefCell<CoreModule>>,
}

impl Module {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn borrow_mut(&self) -> ModuleRef {
        self.module.borrow_mut()
    }

    pub fn function<F>(&self, thunk: F) -> Function<F> {
        Function::new(self.clone(), thunk)
    }

    pub fn build<S>(&self, settings: S) -> errors::Result<Vec<u8>>
    where
        for<'a> ModuleBuilder: TryFrom<(&'a CoreModule, S), Error = errors::Error>,
    {
        build_program(&self.module.borrow() as &CoreModule, settings)
    }

    pub fn build_assembly<S>(&self, settings: S) -> errors::Result<String>
    where
        for<'a> ModuleBuilder: TryFrom<(&'a CoreModule, S), Error = errors::Error>,
    {
        build_program_assembly(&self.module.borrow() as &CoreModule, settings)
    }
}

pub struct NameWrapper(pub(crate) VariableName);

impl<'a> From<&'a str> for NameWrapper {
    fn from(val: &'a str) -> Self {
        Self(VariableName::Named(val.into()))
    }
}

impl From<String> for NameWrapper {
    fn from(val: String) -> Self {
        Self(VariableName::Named(val))
    }
}

impl From<BuiltIn> for NameWrapper {
    fn from(val: BuiltIn) -> Self {
        Self(VariableName::BuiltIn(val))
    }
}

impl From<VariableName> for NameWrapper {
    fn from(val: VariableName) -> Self {
        Self(val)
    }
}

impl From<Option<VariableName>> for NameWrapper {
    fn from(val: Option<VariableName>) -> Self {
        match val {
            Some(val) => val.into(),
            None => Self(VariableName::None),
        }
    }
}

/// Shader attribute
pub trait Input<T> {
    fn input<N>(&self, location: u32, name: N) -> Value<T>
    where
        N: Into<NameWrapper>;
}

/// Shader uniform
pub trait Uniform<T> {
    fn uniform<N>(&self, location: u32, name: N) -> Value<T>
    where
        N: Into<NameWrapper>;
}

/// Shader outputs
pub trait Output<T> {
    fn output<N>(&self, location: u32, name: N, value: Value<T>)
    where
        N: Into<NameWrapper>;
}

pub struct Function<F> {
    pub(crate) module: Module,
    pub(crate) func: FunctionRef,
    pub(crate) thunk: F,
    pub(crate) built: AtomicBool,
}

impl<F: Clone> Clone for Function<F> {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            func: self.func,
            thunk: self.thunk.clone(),
            built: AtomicBool::new(self.built.load(Ordering::SeqCst)),
        }
    }
}

impl<F> Function<F> {
    pub fn new(module: Module, thunk: F) -> Self {
        let func = module.module.borrow_mut().add_function();
        Self {
            func,
            thunk,
            module,
            built: AtomicBool::new(false),
        }
    }

    pub(crate) fn ret_impl<A, R>(
        module: &Module,
        func: FunctionRef,
        source: impl IntoValue<Output = R>,
    ) where
        F: FnOnce<A, Output = Value<R>>,
        Value<R>: IntoValue,
    {
        let src = match source.into_value() {
            Value::Abstract { index, .. } => index,
            source @ Value::Concrete(_) => {
                let module = module.module.borrow_mut();
                let mut graph = RefMut::map(module, |module| &mut module[func]);
                source.get_index(graph)
            }
        };

        let mut module = module.module.borrow_mut();
        let graph = &mut module[func];
        let sink = graph.add_node(Node::Return);
        graph.add_edge(src, sink, 0);
    }

    pub fn ret<A, R>(&self, source: impl IntoValue<Output = R>)
    where
        F: FnOnce<A, Output = Value<R>>,
        Value<R>: IntoValue,
    {
        Self::ret_impl(&self.module, self.func, source);
    }
}

/// Function input
pub trait Parameter<T> {
    fn parameter(&self, location: u32) -> Value<T>;
}
