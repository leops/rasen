#![feature(test, plugin, custom_attribute)]
#![plugin(rasen_plugin)]

extern crate rasen;
extern crate rasen_dsl;
extern crate test;

use rasen_dsl::prelude::*;
use std::f32::consts::PI;
use test::Bencher;

include!("../../tests/plugin.rs");

#[bench]
fn bench_construct_basic_frag(b: &mut Bencher) {
    b.iter(basic_frag_module);
}

#[bench]
fn bench_construct_basic_vert(b: &mut Bencher) {
    b.iter(basic_vert_module);
}

#[bench]
fn bench_call_function(b: &mut Bencher) {
    b.iter(|| func(PI.into()));
}

#[bench]
fn bench_construct_function(b: &mut Bencher) {
    b.iter(functions_module);
}
