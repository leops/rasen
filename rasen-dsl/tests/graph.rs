#![feature(trace_macros)]

extern crate rasen;
#[macro_use]
extern crate rasen_dsl;

mod data;
use data::*;

#[test]
fn test_construct_basic_vert() {
    construct_basic_vert();
}

#[test]
fn test_construct_basic_frag() {
    construct_basic_frag();
}
