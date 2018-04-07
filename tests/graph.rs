fn build_basic_vert() -> Graph {
    let mut graph = Graph::default();

    let pos = graph.add_node(Node::Input(0, TypeName::VEC3));
    let normal = graph.add_node(Node::Input(1, TypeName::VEC3));
    let uv = graph.add_node(Node::Input(2, TypeName::VEC2));

    let projection = graph.add_node(Node::Uniform(0, TypeName::MAT4));
    let view = graph.add_node(Node::Uniform(1, TypeName::MAT4));
    let model = graph.add_node(Node::Uniform(2, TypeName::MAT4));

    let one = graph.add_node(Node::Constant(TypedValue::Float(1.0)));

    let vp = graph.add_node(Node::Multiply);
    let mvp = graph.add_node(Node::Multiply);
    let mul_pos = graph.add_node(Node::Multiply);
    let mul_norm = graph.add_node(Node::Multiply);
    let pos_x = graph.add_node(Node::Extract(0));
    let pos_y = graph.add_node(Node::Extract(1));
    let pos_z = graph.add_node(Node::Extract(2));
    let norm_x = graph.add_node(Node::Extract(0));
    let norm_y = graph.add_node(Node::Extract(1));
    let norm_z = graph.add_node(Node::Extract(2));
    let norm_x2 = graph.add_node(Node::Extract(0));
    let norm_y2 = graph.add_node(Node::Extract(1));
    let norm_z2 = graph.add_node(Node::Extract(2));
    let pos_4 = graph.add_node(Node::Construct(TypeName::VEC4));
    let norm_4 = graph.add_node(Node::Construct(TypeName::VEC4));
    let norm_3 = graph.add_node(Node::Construct(TypeName::VEC3));

    let o_pos = graph.add_node(Node::Output(0, TypeName::VEC4));
    let o_norm = graph.add_node(Node::Output(1, TypeName::VEC3));
    let o_uv = graph.add_node(Node::Output(2, TypeName::VEC2));

    graph.add_edge(projection, vp, 0);
    graph.add_edge(view, vp, 1);
    graph.add_edge(vp, mvp, 0);
    graph.add_edge(model, mvp, 1);

    graph.add_edge(pos, pos_x, 0);
    graph.add_edge(pos, pos_y, 0);
    graph.add_edge(pos, pos_z, 0);
    graph.add_edge(normal, norm_x, 0);
    graph.add_edge(normal, norm_y, 0);
    graph.add_edge(normal, norm_z, 0);

    graph.add_edge(pos_x, pos_4, 0);
    graph.add_edge(pos_y, pos_4, 1);
    graph.add_edge(pos_z, pos_4, 2);
    graph.add_edge(one, pos_4, 3);

    graph.add_edge(mvp, mul_pos, 0);
    graph.add_edge(pos_4, mul_pos, 1);

    graph.add_edge(norm_x, norm_4, 0);
    graph.add_edge(norm_y, norm_4, 1);
    graph.add_edge(norm_z, norm_4, 2);
    graph.add_edge(one, norm_4, 3);
    
    graph.add_edge(model, mul_norm, 0);
    graph.add_edge(norm_4, mul_norm, 1);

    graph.add_edge(mul_norm, norm_x2, 0);
    graph.add_edge(mul_norm, norm_y2, 0);
    graph.add_edge(mul_norm, norm_z2, 0);

    graph.add_edge(norm_x2, norm_3, 0);
    graph.add_edge(norm_y2, norm_3, 1);
    graph.add_edge(norm_z2, norm_3, 2);

    graph.add_edge(mul_pos, o_pos, 0);
    graph.add_edge(norm_3, o_norm, 0);
    graph.add_edge(uv, o_uv, 0);
    
    graph
}

fn build_basic_frag() -> Graph {
    let mut graph = Graph::default();

    let normal = graph.add_node(Node::Input(0, TypeName::VEC3));
    let uv = graph.add_node(Node::Input(1, TypeName::VEC2));

    let material = graph.add_node(Node::Uniform(0, TypeName::SAMPLER2D));

    let min_light = graph.add_node(Node::Constant(TypedValue::Float(0.1)));
    let max_light = graph.add_node(Node::Constant(TypedValue::Float(1.0)));
    let light_dir = graph.add_node(Node::Constant(TypedValue::Vec3(0.3, -0.5, 0.2)));

    let normalize = graph.add_node(Node::Normalize);
    let dot = graph.add_node(Node::Dot);
    let clamp = graph.add_node(Node::Clamp);
    let sample = graph.add_node(Node::Sample);
    let multiply = graph.add_node(Node::Multiply);

    let color = graph.add_node(Node::Output(0, TypeName::VEC4));

    graph.add_edge(normal, normalize, 0);

    graph.add_edge(normalize, dot, 0);
    graph.add_edge(light_dir, dot, 1);

    graph.add_edge(dot, clamp, 0);
    graph.add_edge(min_light, clamp, 1);
    graph.add_edge(max_light, clamp, 2);

    graph.add_edge(material, sample, 0);
    graph.add_edge(uv, sample, 1);

    graph.add_edge(clamp, multiply, 0);
    graph.add_edge(sample, multiply, 1);

    graph.add_edge(multiply, color, 0);
    
    graph
}
