#![feature(test, plugin, custom_attribute)]
#![plugin(rasen_plugin)]

extern crate test;
extern crate rasen;
extern crate rasen_dsl;

use test::Bencher;
use rasen_dsl::prelude::*;

include!("../../tests/plugin.rs");

#[bench]
fn bench_construct_basic_frag(b: &mut Bencher) {
    b.iter(|| basic_frag_module());
}

#[bench]
fn bench_construct_basic_vert(b: &mut Bencher) {
    b.iter(|| basic_vert_module());
}

#[bench]
fn bench_call_function(b: &mut Bencher) {
    b.iter(|| func(3.14f32.into()));
}

#[bench]
fn bench_construct_function(b: &mut Bencher) {
    b.iter(|| functions_module());
}
