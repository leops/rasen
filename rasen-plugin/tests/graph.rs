#![feature(plugin, custom_attribute, try_from)]
#![plugin(rasen_plugin)]

extern crate rasen;
extern crate rasen_dsl;
#[macro_use]
extern crate pretty_assertions;
extern crate rspirv;

use rasen_dsl::prelude::*;
use std::f32::consts::PI;

include!("../../tests/plugin.rs");
include!("../../tests/update.rs");

#[test]
fn gen_basic_vert() {
    let module = basic_vert_module();
    let assembly = module
        .build_assembly(Settings {
            mod_type: ShaderType::Vertex,
            uniforms_name: Some(String::from("Uniforms")),
        })
        .unwrap();
    check_or_update!(assembly, "../../tests/basic-plugin.vert.spvasm");
}

#[test]
fn gen_basic_frag() {
    let module = basic_frag_module();
    let assembly = module
        .build_assembly(Settings {
            mod_type: ShaderType::Fragment,
            uniforms_name: Some(String::from("Uniforms")),
        })
        .unwrap();
    check_or_update!(assembly, "../../tests/basic-plugin.frag.spvasm");
}

#[test]
#[allow(clippy::float_cmp)]
fn call_functions() {
    let result = func(PI.into());
    let result = match result {
        Value::Concrete(v) => v,
        _ => panic!("result is not concrete"),
    };
    assert_eq!(result, PI);
}

#[test]
fn gen_functions() {
    let module = functions_module();
    let assembly = module
        .build_assembly(Settings {
            mod_type: ShaderType::Vertex,
            uniforms_name: Some(String::from("Uniforms")),
        })
        .unwrap();
    check_or_update!(assembly, "../../tests/functions.spvasm");
}
