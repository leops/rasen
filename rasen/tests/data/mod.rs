use rasen::*;
use rasen::TypedValue::*;

pub fn construct_basic_vert() -> Graph {
    rasen_graph! {
        a_pos = Input(0, TypeName::VEC3);
        Output(0, TypeName::VEC4) {
            Multiply {
                Multiply {
                    Multiply {
                        Uniform(0, TypeName::MAT4)
                        Uniform(1, TypeName::MAT4)
                    }
                    Uniform(2, TypeName::MAT4)
                }
                Construct(TypeName::VEC4) {
                    Extract(0) { a_pos }
                    Extract(1) { a_pos }
                    Extract(2) { a_pos }
                    Constant(Float(1.0))
                }
            }
        };

        a_normal = Input(1, TypeName::VEC3);
        Output(1, TypeName::VEC4) {
            Multiply {
                Uniform(2, TypeName::MAT4)
                Construct(TypeName::VEC4) {
                    Extract(0) { a_normal }
                    Extract(1) { a_normal }
                    Extract(2) { a_normal }
                    Constant(Float(0.0))
                }
            }
        };

        Output(2, TypeName::VEC2) {
            Input(2, TypeName::VEC2)
        };
    }
}

pub fn construct_basic_frag() -> Graph {
    rasen_graph! {
        Output(0, TypeName::VEC4) {
            Multiply {
                Clamp {
                    Dot {
                        Normalize {
                            Input(0, TypeName::VEC3)
                        }
                        Constant(Vec3(0.3, -0.5, 0.2))
                    }
                    Constant(Float(0.1))
                    Constant(Float(1.0))
                }
                Constant(Vec4(0.25, 0.625, 1.0, 1.0))
            }
        };
    }
}
