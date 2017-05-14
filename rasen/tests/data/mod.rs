use rasen_dsl::prelude::*;

#[shader]
pub fn basic_vert(a_pos: Value<Vec3>, a_normal: Value<Vec3>, a_uv: Value<Vec2>, projection: Value<Mat4>, view: Value<Mat4>, model: Value<Mat4>) -> (Value<Vec4>, Value<Vec4>, Value<Vec2>) {
    let mvp = projection * view * model.clone();

    let v_pos = mvp * vec4(index(&a_pos, 0), index(&a_pos, 1), index(&a_pos, 2), 1.0f32);
    let v_norm = model * vec4(index(&a_normal, 0), index(&a_normal, 1), index(&a_normal, 2), 1.0f32);

    (v_pos, v_norm, a_uv)
}

#[shader]
pub fn basic_frag(a_input: Value<Vec3>) -> Value<Vec4> {
    let normal = normalize(a_input);
    let light = vec3(0.3f32, -0.5f32, 0.2f32);
    let color = vec4(0.25f32, 0.625f32, 1.0f32, 1.0f32);

    clamp(dot(normal, light), 0.1f32, 1.0f32) * color
}
