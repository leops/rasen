extern crate rasen;

use rasen::prelude::*;

include!("../../tests/graph.rs");

static REF_VERT: &'static [u8] = include_bytes!("../../tests/basic.vert.spv");
static REF_FRAG: &'static [u8] = include_bytes!("../../tests/basic.frag.spv");

#[test]
fn test_build_basic_vert() {
    let graph = build_basic_vert();
    let bytecode = build_program(&graph, ShaderType::Vertex).unwrap();
    assert_eq!(bytecode, REF_VERT);
}

#[test]
fn test_build_basic_frag() {
    let graph = build_basic_frag();
    let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
    assert_eq!(bytecode, REF_FRAG);
}
