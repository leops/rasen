#![feature(plugin, custom_attribute, try_from)]
#![plugin(rasen_plugin)]

extern crate rasen;
extern crate rasen_dsl;
extern crate rspirv;
extern crate insta;

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

    assert_spirv_snapshot_matches!("basic-plugin.vert", assembly);
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

    assert_spirv_snapshot_matches!("basic-plugin.frag", assembly);
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

    assert_spirv_snapshot_matches!("functions", assembly);
}
