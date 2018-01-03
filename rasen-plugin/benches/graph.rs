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
    b.iter(|| basic_frag_shader());
}

#[bench]
fn bench_construct_basic_vert(b: &mut Bencher) {
    b.iter(|| basic_vert_shader());
}
