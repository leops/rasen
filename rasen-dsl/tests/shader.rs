#![feature(specialization, conservative_impl_trait)]

extern crate rasen;
extern crate rasen_dsl;

use rasen_dsl::prelude::*;

#[test]
fn test_dsl() {
    let shader = Shader::new();

    let normal: Value<Vec3> = normalize(shader.input(0));
    let light = Vec3(0.3, -0.5, 0.2);
    let color = Vec4(0.25, 0.625, 1.0, 1.0);

    let res = clamp(dot(normal, light), 0.1f32, 1.0f32) * color;
    shader.output(0, res);

    shader.build(ShaderType::Fragment).unwrap();
}
