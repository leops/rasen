use petgraph::graph::NodeIndex;
use types::TypeName;
use node::Node;
use error_chain;

error_chain! {
    errors {
        WrongArgumentsCount(actual: usize, expected: usize) {
            description("wrong number of arguments")
            display("got {} arguments, expected {}", actual, expected)
        }
        BadArguments(args: Box<[&'static TypeName]>) {
            description("bad arguments")
            display("bad arguments: {:?}", args)
        }
        UnsupportedConstant(ty: &'static TypeName) {
            description("unsupported constant type")
            display("unsupported constant type {:?}", ty)
        }
        BuildError(node: &'static str, id: usize) {
            description("build error")
            display("compilation failed at {} node with id {}", node, id)
        }
        CyclicGraph {
            description("graph is cyclic")
            display("graph is cyclic")
        }
    }
}

#[inline]
pub fn build_error<'a>(node: &'a Node, id: NodeIndex<u32>) -> impl FnOnce(Error) -> Error {
    move |err: Error| match err.kind() {
        &ErrorKind::BuildError(..) => err,
        _ => {
            let state = error_chain::State::new::<Error>(Box::new(err));
            error_chain::ChainedError::new(
                ErrorKind::BuildError(node.to_string(), id.index()).into(),
                state
            )
        }
    }
}
