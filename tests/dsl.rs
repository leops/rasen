pub fn basic_vert<C: Context>(
    a_pos: Value<C, Vec3>,
    a_normal: Value<C, Vec3>,
    a_uv: Value<C, Vec2>,
    projection: Value<C, Mat4>,
    view: Value<C, Mat4>,
    model: Value<C, Mat4>,
) -> (Value<C, Vec4>, Value<C, Vec4>, Value<C, Vec2>) {
    let mvp = projection * view * model;

    let v_pos = mvp * vec4(index(a_pos, 0), index(a_pos, 1), index(a_pos, 2), 1.0f32);

    let v_norm = model
        * vec4(
            index(a_normal, 0),
            index(a_normal, 1),
            index(a_normal, 2),
            1.0f32,
        );

    (v_pos, v_norm, a_uv)
}

pub fn basic_frag<C: Context>(
    a_normal: Value<C, Vec3>,
    a_uv: Value<C, Vec2>,
    material: Value<C, Sampler>,
) -> Value<C, Vec4> {
    let normal = normalize(a_normal);
    let light = vec3(0.3f32, -0.5f32, 0.2f32);
    let color = sample(material, a_uv);

    clamp(dot(normal, light), 0.1f32, 1.0f32) * color
}

fn func<C: Context>(input: Value<C, f32>) -> Value<C, f32> {
    input
}

pub fn functions<C: Context>(a_input: Value<C, f32>) -> Value<C, f32> {
    func(a_input)
}
