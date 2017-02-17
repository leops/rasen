#![feature(trace_macros)]

#[macro_use]
extern crate rasen;

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
