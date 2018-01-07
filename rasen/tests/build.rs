extern crate rasen;

use rasen::prelude::*;

include!("../../tests/graph.rs");

static REF_VERT: &'static str = include_str!("../../tests/basic.vert.spvasm");
static REF_FRAG: &'static str = include_str!("../../tests/basic.frag.spvasm");

#[test]
fn test_build_basic_vert() {
    let graph = build_basic_vert();
    let assembly = build_program_assembly(&graph, ShaderType::Vertex).unwrap();
    assert_eq!(assembly, REF_VERT);
}

#[test]
fn test_build_basic_frag() {
    let graph = build_basic_frag();
    let assembly = build_program_assembly(&graph, ShaderType::Fragment).unwrap();
    assert_eq!(assembly, REF_FRAG);
}
