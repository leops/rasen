#![feature(test)]

extern crate rasen;
extern crate rasen_dsl;
extern crate test;

mod data;

use test::Bencher;
use data::*;
use rasen::prelude::*;

#[bench]
fn bench_build_basic_vert(b: &mut Bencher) {
    let graph = construct_basic_vert();
    b.iter(||
        graph.build(ShaderType::Vertex).unwrap()
    );
}

#[bench]
fn bench_build_basic_frag(b: &mut Bencher) {
    let graph = construct_basic_frag();
    b.iter(||
        graph.build(ShaderType::Fragment).unwrap()
    );
}
