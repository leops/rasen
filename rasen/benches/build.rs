#![feature(test)]

extern crate rasen;
extern crate test;

use rasen::prelude::*;
use test::Bencher;

include!("../../tests/graph.rs");

#[bench]
fn bench_build_basic_vert(b: &mut Bencher) {
    let graph = build_basic_vert();
    b.iter(|| {
        ModuleBuilder::from_graph(
            &graph,
            Settings {
                mod_type: ShaderType::Vertex,
                uniforms_name: Some(String::from("Uniforms")),
            },
        )
        .unwrap()
        .build()
        .unwrap()
    });
}

#[bench]
fn bench_build_basic_frag(b: &mut Bencher) {
    let graph = build_basic_frag();
    b.iter(|| {
        ModuleBuilder::from_graph(
            &graph,
            Settings {
                mod_type: ShaderType::Fragment,
                uniforms_name: Some(String::from("Uniforms")),
            },
        )
        .unwrap()
        .build()
        .unwrap()
    });
}
