#![feature(test)]

extern crate test;
extern crate rasen;

use test::Bencher;
use rasen::prelude::*;

include!("../../tests/graph.rs");

#[bench]
fn bench_build_basic_vert(b: &mut Bencher) {
    let graph = build_basic_vert();
    b.iter(||
        build_program(&graph, ShaderType::Vertex).unwrap()
    );
}

#[bench]
fn bench_build_basic_frag(b: &mut Bencher) {
    let graph = build_basic_frag();
    b.iter(||
        build_program(&graph, ShaderType::Fragment).unwrap()
    );
}
