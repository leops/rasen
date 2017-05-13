#![feature(trace_macros)]

extern crate rasen;
extern crate rasen_dsl;

mod data;
use data::*;
use rasen_dsl::prelude::*;

#[test]
fn test_run_basic_vert() {
    let a_pos = vec3(1.0f32, 2.0f32, 3.0f32);
    let a_normal = vec3(0.0f32, 1.0f32, 0.0f32);
    let a_uv = vec2(0.5f32, 0.5f32);

    let projection = Mat4([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ]);
    let view = Mat4([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ]);
    let model = Mat4([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ]);

    let (v_pos, v_norm, a_uv) = basic_vert(
        a_pos, a_normal, a_uv,
        projection.into(),
        view.into(),
        model.into()
    );
}

#[test]
fn test_construct_basic_frag() {
    construct_basic_frag();
}
