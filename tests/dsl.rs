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
    material: Value<C, Sampler<Vec4>>,
) -> Value<C, Vec4> {
    let normal = normalize(a_normal);
    let light = vec3(0.3f32, -0.5f32, 0.2f32);
    let color = sample(material, a_uv);

    clamp(dot(normal, light), 0.1f32, 1.0f32) * color
}

fn extract_color<C: Context>(input: Value<C, Vec4>) -> Value<C, Vec3> {
    vec3(
        index(input, 0),
        index(input, 1),
        index(input, 2),
    )
}

pub fn composite_ycrcb<C: Context>(
    a_uv: Value<C, Vec2>,
    luma_sampler: Value<C, Sampler<Vec4>>,
    chroma_sampler: Value<C, Sampler<Vec4>>,
    scene_sampler: Value<C, Sampler<Vec4>>,
) -> Value<C, Vec4> {
    let ycbcr_to_rgb = mat4(
        vec4( 1.0000f32,  1.0000f32,  1.0000f32,  0.0000f32),
        vec4( 0.0000f32, -0.3441f32,  1.7720f32,  0.0000f32),
        vec4( 1.4020f32, -0.7141f32,  0.0000f32,  0.0000f32),
        vec4(-0.7010f32,  0.5291f32, -0.8860f32,  1.0000f32),
    );

    let luma = sample(luma_sampler, a_uv);
    let chroma = sample(chroma_sampler, a_uv);
    let scene = sample(scene_sampler, a_uv);

    let input = vec4(
        index(luma, 0),
        index(chroma, 0),
        index(chroma, 1),
        1.0f32,
    );
    let output = ycbcr_to_rgb * input;

    let alpha = index(scene, 3);
    let composite = mix(
        extract_color(output),
        extract_color(scene),
        vec3(alpha, alpha, alpha),
    );

    vec4(
        index(composite, 0),
        index(composite, 1),
        index(composite, 2),
        1.0f32,
    )
}

fn func<C: Context>(input: Value<C, f32>) -> Value<C, f32> {
    input
}

pub fn functions<C: Context>(a_input: Value<C, f32>) -> Value<C, f32> {
    func(a_input)
}
