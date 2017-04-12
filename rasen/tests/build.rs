extern crate rasen;
#[macro_use]
extern crate rasen_dsl;

use rasen::*;

mod data;
use data::*;

static REF_VERT: &'static [u8] = include_bytes!("data/basic.vert.spv");
static REF_FRAG: &'static [u8] = include_bytes!("data/basic.frag.spv");

#[test]
fn test_build_basic_vert() {
    let graph = construct_basic_vert();
    let bytecode = build_program(&graph, ShaderType::Vertex).unwrap();
    assert_eq!(bytecode, REF_VERT);
}

#[test]
fn test_build_basic_frag() {
    let graph = construct_basic_frag();
    let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
    assert_eq!(bytecode, REF_FRAG);
}
