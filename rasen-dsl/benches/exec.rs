#![feature(test)]

extern crate rasen;
extern crate rasen_dsl;
extern crate test;

use rasen_dsl::prelude::*;
use std::f32::consts::PI;
use test::Bencher;

include!("../../tests/dsl.rs");

#[bench]
fn bench_run_basic_frag(b: &mut Bencher) {
    b.iter(|| {
        basic_frag(
            vec3(0.0f32, 1.0f32, 0.0f32),
            vec2(0.0f32, 0.0f32),
            Value::Concrete(Sampler(Vec4(0.25f32, 0.625f32, 1.0f32, 1.0f32))),
        );
    });
}

#[bench]
fn bench_run_basic_vert(b: &mut Bencher) {
    b.iter(|| {
        let a_pos = vec3(1.0f32, 2.0f32, 3.0f32);
        let a_normal = vec3(0.0f32, 1.0f32, 0.0f32);
        let a_uv = vec2(0.5f32, 0.5f32);

        #[rustfmt::skip]
        let projection = Mat4([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ]);
        #[rustfmt::skip]
        let view = Mat4([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ]);
        #[rustfmt::skip]
        let model = Mat4([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ]);

        basic_vert(
            a_pos,
            a_normal,
            a_uv,
            projection.into(),
            view.into(),
            model.into(),
        )
    });
}

#[bench]
fn bench_run_functions(b: &mut Bencher) {
    b.iter(|| {
        functions(PI.into());
    });
}
