extern crate insta;
extern crate rasen;
extern crate rspirv;

use rasen::prelude::*;

include!("../../tests/update.rs");

#[test]
fn test_build_loop() {
    let mut module = Module::default();

    let func_cond = module.add_function();

    {
        let graph = &mut module[func_cond];
        let input = graph.add_node(Node::Parameter(0, TypeName::FLOAT));
        let cmp = graph.add_node(Node::Greater);
        let ten = graph.add_node(Node::Constant(TypedValue::Float(10.0)));
        let output = graph.add_node(Node::Return);

        graph.add_edge(input, cmp, 0);
        graph.add_edge(ten, cmp, 1);
        graph.add_edge(cmp, output, 0);
    }

    let func_body = module.add_function();

    {
        let graph = &mut module[func_body];
        let input = graph.add_node(Node::Parameter(0, TypeName::FLOAT));
        let add = graph.add_node(Node::Add);
        let one = graph.add_node(Node::Constant(TypedValue::Float(1.0)));
        let output = graph.add_node(Node::Return);

        graph.add_edge(input, add, 0);
        graph.add_edge(one, add, 1);
        graph.add_edge(add, output, 0);
    }

    {
        let graph = &mut module.main;
        let input = graph.add_node(Node::Input(
            0,
            TypeName::FLOAT,
            VariableName::Named(String::from("i_value")),
        ));
        let reduce = graph.add_node(Node::Loop(func_cond, func_body));
        let output = graph.add_node(Node::Output(
            0,
            TypeName::FLOAT,
            VariableName::Named(String::from("o_value")),
        ));

        graph.add_edge(input, reduce, 0);
        graph.add_edge(reduce, output, 0);
    }

    let builder = ModuleBuilder::from_module(
        &module,
        Settings {
            mod_type: ShaderType::Fragment,
            uniforms_name: Some(String::from("uniforms")),
        },
    )
    .expect("from_module");

    let assembly = builder.into_assembly().expect("build");
    assert_spirv_snapshot_matches!("loop.frag", assembly);
}
