extern crate rasen;
#[macro_use]
extern crate pretty_assertions;
extern crate rspirv;

use rasen::prelude::*;

include!("../../tests/graph.rs");
include!("../../tests/update.rs");

#[test]
fn test_build_basic_vert() {
    let graph = build_basic_vert();
    let assembly = build_module(&graph, ShaderType::Vertex).unwrap();
    check_or_update!(assembly, "../../tests/basic.vert.spvasm");
}

#[test]
fn test_build_basic_frag() {
    let graph = build_basic_frag();
    let assembly = build_module(&graph, ShaderType::Fragment).unwrap();
    check_or_update!(assembly, "../../tests/basic.frag.spvasm");
}

#[test]
fn test_build_function() {
    let mut module = Module::default();

    let func = module.add_function();

    {
        let graph = &mut module[func];
        let input = graph.add_node(Node::Parameter(0, TypeName::FLOAT));
        let output = graph.add_node(Node::Return);
        graph.add_edge(input, output, 0);
    }

    {
        let graph = &mut module.main;
        let input = graph.add_node(Node::Input(0, TypeName::FLOAT, VariableName::Named(String::from("a_input"))));
        let call = graph.add_node(Node::Call(func));
        let output = graph.add_node(Node::Output(0, TypeName::FLOAT, VariableName::None));
        graph.add_edge(input, call, 0);
        graph.add_edge(call, output, 0);
    }

    let assembly = build_module(&module, ShaderType::Vertex).unwrap();
    check_or_update!(assembly, "../../tests/functions.spvasm");
}
