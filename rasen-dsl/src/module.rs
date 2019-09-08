//! Module builder utility

use std::{
    rc::Rc,
    cell::RefCell,
    ops::{Fn, FnMut, FnOnce},
};

use crate::{
    context::{
        parse::{Parse, ParseNode},
        Container,
    },
    types::AsTypeName,
    value::Value,
};
use rasen::{
    module::FunctionRef,
    prelude::{Graph, Module as ModuleImpl, Node, VariableName, BuiltIn},
};

type Shared<T> = Rc<RefCell<T>>;
type BuilderContext = Option<(Shared<ModuleImpl>, Option<FunctionRef>)>;

thread_local! {
    static CONTEXT: RefCell<BuilderContext> = RefCell::new(None);
}

pub(crate) fn with_graph<T>(thunk: impl FnOnce(&mut Graph) -> T) -> T {
    CONTEXT.with(|ctx| {
        let ctx = ctx.borrow();
        let (module, func) = ctx
            .as_ref()
            .expect("Module builder called outside of Module::with");

        let mut module = module.borrow_mut();
        if let Some(func) = func {
            thunk(&mut module[*func])
        } else {
            thunk(&mut module.main)
        }
    })
}

fn using_module<T>(push: impl FnOnce(&mut BuilderContext) -> (), code: impl FnOnce() -> (), pop: impl FnOnce(&mut BuilderContext) -> T) -> T {
    CONTEXT.with(move |cell| {
        let mut ctx = cell.borrow_mut();
        push(&mut ctx as &mut BuilderContext);
    });

    code();

    CONTEXT.with(move |cell| {
        let mut ctx = cell.borrow_mut();
        pop(&mut ctx as &mut BuilderContext)
    })
}

pub trait IntoVariableName {
    fn into_variable_name(self) -> VariableName;
}

impl IntoVariableName for BuiltIn {
    fn into_variable_name(self) -> VariableName {
        VariableName::BuiltIn(self)
    }
}

impl IntoVariableName for String {
    fn into_variable_name(self) -> VariableName {
        VariableName::Named(self)
    }
}

impl<'a> IntoVariableName for &'a str {
    fn into_variable_name(self) -> VariableName {
        VariableName::Named(self.into())
    }
}

impl<T: IntoVariableName> IntoVariableName for Option<T> {
    fn into_variable_name(self) -> VariableName {
        if let Some(inner) = self {
            inner.into_variable_name()
        } else {
            VariableName::None
        }
    }
}

pub struct Module;

impl Module {
    pub fn build(thunk: impl FnOnce(&Self) -> ()) -> ModuleImpl {
        let module = Rc::new(RefCell::new(
            ModuleImpl::default()
        ));

        using_module(
            |ctx| {
                debug_assert!(ctx.is_none(), "Module::build called recursively");
                *ctx = Some((module, None));
            },
            || {
                thunk(&Module);
            },
            |ctx| {
                let value = ctx.take();

                let (module, func) = value.expect("Builder is missing in thread local key");
                debug_assert!(func.is_none(), "Module builder unwrapped from function context");
                
                let module = Rc::try_unwrap(module).expect("Module builder has live references");
                module.into_inner()
            },
        )
    }

    pub fn input<T>(&self, index: u32, name: impl IntoVariableName) -> Value<Parse, T>
    where
        T: AsTypeName,
        Parse: Container<T, Value = ParseNode>,
    {
        with_graph(|graph| {
            Value(graph.add_node(Node::Input(index, T::TYPE_NAME, name.into_variable_name())))
        })
    }

    pub fn uniform<T>(&self, index: u32, name: impl IntoVariableName) -> Value<Parse, T>
    where
        T: AsTypeName,
        Parse: Container<T, Value = ParseNode>,
    {
        with_graph(|graph| {
            Value(graph.add_node(Node::Uniform(index, T::TYPE_NAME, name.into_variable_name())))
        })
    }

    pub fn output<T>(&self, index: u32, name: impl IntoVariableName, value: Value<Parse, T>)
    where
        T: AsTypeName,
        Parse: Container<T, Value = ParseNode>,
    {
        with_graph(|graph| {
            let node = graph.add_node(Node::Output(index, T::TYPE_NAME, name.into_variable_name()));
            graph.add_edge(value.0, node, 0);
        });
    }

    pub fn function<F, A, R>(&self, function: F) -> impl Fn<A, Output = Value<Parse, R>>
    where
        F: FnOnce<A, Output = Value<Parse, R>>,
        A: FnArgs,
        Parse: Container<R, Value = ParseNode>,
    {
        let func = using_module(
            |ctx| {
                if let Some((module, func)) = ctx {
                    debug_assert!(func.is_none(), "Cannot build functions recursively");
                    let mut module = module.borrow_mut();
                    *func = Some(module.add_function());
                } else {
                    panic!("Function builder called outside of module builder");
                }
            },
            || {
                let res = function.call_once(A::create());
                with_graph(|graph| {
                    let node = graph.add_node(Node::Return);
                    graph.add_edge(res.0, node, 0);
                });
            },
            |ctx| {
                if let Some((_module, func)) = ctx {
                    let func = func.take();
                    func.expect("Function builder unwrapped outside of function context")
                } else {
                    panic!("Function builder unwrapped outside of module builder")
                }
            },
        );

        FnWrapper(move |args: A| -> Value<Parse, R> {
            Value(args.call(func))
        })
    }
}

pub trait FnArgs {
    fn create() -> Self;
    fn call(self, func: FunctionRef) -> ParseNode;
}

include! {
    concat!(env!("OUT_DIR"), "/module.rs")
}

struct FnWrapper<F>(F);

impl<F: Fn<(A,)>, A> Fn<A> for FnWrapper<F> {
    extern "rust-call" fn call(&self, args: A) -> Self::Output {
        (self.0)(args)
    }
}

impl<F: Fn<(A,)>, A> FnMut<A> for FnWrapper<F> {
    extern "rust-call" fn call_mut(&mut self, args: A) -> Self::Output {
        (self.0)(args)
    }
}

impl<F: Fn<(A,)>, A> FnOnce<A> for FnWrapper<F> {
    type Output = F::Output;

    extern "rust-call" fn call_once(self, args: A) -> Self::Output {
        (self.0)(args)
    }
}
