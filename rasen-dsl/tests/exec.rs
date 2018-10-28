#![feature(try_from)]

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
        projection.into(),
        view.into(),
        model.into(),
    );

    let Vec4(v_pos_x, v_pos_y, v_pos_z, v_pos_w) = match v_pos {
        Value::Concrete(v) => v,
        _ => panic!("v_pos is not concrete"),
    };
    assert_eq!((v_pos_x, v_pos_y, v_pos_z, v_pos_w), (1.0, 2.0, 3.0, 1.0));

    let Vec4(v_norm_x, v_norm_y, v_norm_z, v_norm_w) = match v_norm {
        Value::Concrete(v) => v,
        _ => panic!("v_norm is not concrete"),
    };
    assert_eq!(
        (v_norm_x, v_norm_y, v_norm_z, v_norm_w),
        (0.0, 1.0, 0.0, 1.0)
    );

    let Vec2(v_uv_x, v_uv_y) = match v_uv {
        Value::Concrete(v) => v,
        _ => panic!("v_uv is not concrete"),
    };
    assert_eq!((v_uv_x, v_uv_y), (0.5, 0.5));
}

#[test]
fn test_run_basic_frag() {
    let color = basic_frag(
        vec3(0.0f32, 1.0f32, 0.0f32),
        vec2(0.0f32, 0.0f32),
        Value::Concrete(Sampler(Vec4(0.25f32, 0.625f32, 1.0f32, 1.0f32))),
    );

    let Vec4(color_r, color_g, color_b, color_a) = match color {
        Value::Concrete(v) => v,
        _ => panic!("color is not concrete"),
    };
    assert_eq!(
        (color_r, color_g, color_b, color_a),
        (0.025, 0.0625, 0.1, 0.1)
    );
}

#[test]
#[allow(clippy::float_cmp)]
fn test_run_functions() {
    let result = functions(PI.into());
    let result = match result {
        Value::Concrete(v) => v,
        _ => panic!("result is not concrete"),
    };
    assert_eq!(result, PI);
}
