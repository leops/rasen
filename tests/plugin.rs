#[rasen(module)]
#[allow(clippy::needless_pass_by_value)]
pub fn basic_vert(
    a_pos: Value<Vec3>,
    a_normal: Value<Vec3>,
    a_uv: Value<Vec2>,
    projection: Value<Mat4>,
    view: Value<Mat4>,
    model: Value<Mat4>,
) -> (Value<Vec4>, Value<Vec3>, Value<Vec2>) {
    let mvp = projection * view * model.clone();

    let v_pos = mvp * vec4!(a_pos, 1.0f32);
    let v_norm = model * vec4!(a_normal, 1.0f32);

    (v_pos, idx!(v_norm, xyz), a_uv)
}

#[rasen(module)]
pub fn basic_frag(
    a_normal: Value<Vec3>,
    a_uv: Value<Vec2>,
    material: Value<Sampler<Vec4>>,
) -> Value<Vec4> {
    let normal = normalize(a_normal);
    let light = vec3(0.3f32, -0.5f32, 0.2f32);
    let color = sample(material, a_uv);

    clamp(dot(normal, light), 0.1f32, 1.0f32) * color
}

#[rasen(function)]
fn func(input: Value<Float>) -> Value<Float> {
    input
}

#[rasen(module)]
pub fn functions(a_input: Value<Float>) -> Value<Float> {
    func(a_input)
}
