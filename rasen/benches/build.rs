#![feature(test)]

extern crate rasen;
#[macro_use]
extern crate rasen_dsl;
extern crate test;

mod data;

use test::Bencher;
use data::*;
use rasen::*;

#[bench]
fn bench_build_basic_vert(b: &mut Bencher) {
    let graph = construct_basic_vert();
    b.iter(||
        build_program(&graph, ShaderType::Vertex).unwrap()
    );
}

#[bench]
fn bench_build_basic_frag(b: &mut Bencher) {
    let graph = construct_basic_frag();
    b.iter(||
        build_program(&graph, ShaderType::Fragment).unwrap()
    );
}
