rasen
================
Generate SPIR-V bytecode from an operation graph (heavy WIP)

```rust
#[macro_use]
extern crate rasen;

use rasen::*;

fn main() {
    let graph = rasen_graph! {
        nodes {
            // A vec3 input at location 0
            normal = Node::Input(0, TypeName::VEC3),

            // Some ambient light constants
            min_light = Node::Constant(TypedValue::Float(0.1)),
            max_light = Node::Constant(TypedValue::Float(1.0)),
            light_dir = Node::Constant(TypedValue::Vec3(0.3, -0.5, 0.2)),

            // The Material color (also a constant)
            mat_color = Node::Constant(TypedValue::Vec4(0.25, 0.625, 1.0, 1.0)),

            // Some usual function calls
            normalize = Node::Normalize,
            dot = Node::Dot,
            clamp = Node::Clamp,
            multiply = Node::Multiply,

            // And a vec4 output at location 0
            color = Node::Output(0, TypeName::VEC4)
        }

        edges {
            // Normalize the normal
            normalize(normal),

            // Compute the dot product of the surface normal and the light direction
            dot(normalize, light_dir),

            // Restrict the result into the ambient light range
            clamp(dot, min_light, max_light),

            // Multiply the light intensity by the surface color
            multiply(clamp, mat_color),

            // Write the result to the output
            color(multiply)
        }
    };

    let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
    // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
}
```
