#![feature(plugin, custom_attribute)]
#![plugin(rasen_plugin)]

extern crate rasen;
extern crate rasen_dsl;

use rasen_dsl::prelude::*;

include!("../../tests/plugin.rs");

static REF_VERT: &'static str = include_str!("../../tests/basic.vert.spvasm");
static REF_FRAG: &'static str = include_str!("../../tests/basic.frag.spvasm");

#[test]
fn test_build_basic_vert() {
    let shader = basic_vert_shader();
    let assembly = shader.build_assembly(ShaderType::Vertex).unwrap();
    assert_eq!(assembly, REF_VERT);
}

#[test]
fn test_build_basic_frag() {
    let shader = basic_frag_shader();
    let assembly = shader.build_assembly(ShaderType::Fragment).unwrap();
    assert_eq!(assembly, REF_FRAG);
}
