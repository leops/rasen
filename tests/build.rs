#[macro_use]
extern crate rasen;
use rasen::*;

mod data;
use data::*;

#[test]
fn test_build_basic_vert() {
    let graph = construct_basic_vert();
    build_program(&graph, ShaderType::Vertex).unwrap();
}

#[test]
fn test_build_basic_frag() {
    let graph = construct_basic_frag();
    build_program(&graph, ShaderType::Fragment).unwrap();
}
