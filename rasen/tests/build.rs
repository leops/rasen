#![feature(plugin, custom_attribute)]
#![plugin(rasen_plugin)]

extern crate rasen;
extern crate rasen_dsl;

use rasen::prelude::*;

mod data;
use data::*;

static REF_VERT: &'static [u8] = include_bytes!("data/basic.vert.spv");
static REF_FRAG: &'static [u8] = include_bytes!("data/basic.frag.spv");

#[test]
fn test_build_basic_vert() {
    let shader = basic_vert_shader();
    let bytecode = shader.build(ShaderType::Vertex).unwrap();
    assert_eq!(bytecode, REF_VERT);
}

#[test]
fn test_build_basic_frag() {
    let shader = basic_frag_shader();
    let bytecode = shader.build(ShaderType::Fragment).unwrap();
    assert_eq!(bytecode, REF_FRAG);
}
