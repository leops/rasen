use std::f32::consts::PI;

extern crate rasen;
extern crate rasen_dsl;
#[macro_use]
extern crate pretty_assertions;
extern crate rspirv;

use rasen_dsl::prelude::*;

include!("../../tests/dsl.rs");

#[test]
fn test_run_basic_vert() {
    let a_pos = vec3(1.0f32, 2.0f32, 3.0f32);
    let a_normal = vec3(0.0f32, 1.0f32, 0.0f32);
    let a_uv = vec2(0.5f32, 0.5f32);

    #[rustfmt::skip]
    let projection = Mat4([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);
    #[rustfmt::skip]
    let view = Mat4([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);
    #[rustfmt::skip]
    let model = Mat4([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    let (v_pos, v_norm, v_uv) = basic_vert(
        a_pos,
        a_normal,
        a_uv,
        Value::of(projection),
        Value::of(view),
        Value::of(model),
    );

    let Vec4(v_pos) = v_pos.read();
    assert_eq!(v_pos, [1.0, 2.0, 3.0, 1.0]);

    let Vec4(v_norm) = v_norm.read();
    assert_eq!(
        v_norm,
        [0.0, 1.0, 0.0, 1.0]
    );

    let Vec2(v_uv) = v_uv.read();
    assert_eq!(v_uv, [0.5, 0.5]);
}

#[test]
fn test_run_basic_frag() {
    let color = basic_frag(
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec2(0.0f32, 0.0f32),
        Value::of(Sampler(Vec4([0.25f32, 0.625f32, 1.0f32, 1.0f32]))),
    );

    let Vec4(color) = color.read();
    assert_eq!(
        color,
        [0.025, 0.0625, 0.1, 0.1]
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_run_functions() {
    let result = functions(Value::of(PI));
    let result = result.read();
    assert_eq!(result, PI);
}
