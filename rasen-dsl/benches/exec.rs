#![feature(test)]

extern crate rasen;
extern crate rasen_dsl;
extern crate test;

mod data;

use rasen_dsl::prelude::*;

use test::Bencher;
use data::*;

#[bench]
fn bench_run_basic_frag(b: &mut Bencher) {
    b.iter(|| {
        basic_frag(vec3(0.0f32, 1.0f32, 0.0f32));
    });
}

#[bench]
fn bench_run_basic_vert(b: &mut Bencher) {
    b.iter(|| {
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

        basic_vert(
            a_pos, a_normal, a_uv,
            projection.into(),
            view.into(),
            model.into()
        )
    });
}
