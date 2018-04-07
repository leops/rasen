#![feature(plugin, custom_attribute)]
#![plugin(rasen_plugin)]

extern crate rasen;
extern crate rasen_dsl;

use rasen_dsl::prelude::*;

include!("../../tests/plugin.rs");
include!("../../tests/update.rs");

#[test]
fn test_build_basic_vert() {
    let module = basic_vert_module();
    let assembly = module.build_assembly(ShaderType::Vertex).unwrap();
    check_or_update!(assembly, "../../tests/basic.vert.spvasm");
}

#[test]
fn test_build_basic_frag() {
    let module = basic_frag_module();
    let assembly = module.build_assembly(ShaderType::Fragment).unwrap();
    check_or_update!(assembly, "../../tests/basic.frag.spvasm");
}

#[test]
fn test_call_functions() {
    let result = func(3.14f32.into());
    let result = match result {
        Value::Concrete(v) => v,
        _ => panic!("result is not concrete"),
    };
    assert_eq!(result, 3.14f32);
}

#[test]
fn test_build_functions() {
    let module = functions_module();
    let assembly = module.build_assembly(ShaderType::Vertex).unwrap();
    check_or_update!(assembly, "../../tests/functions.spvasm");
}
