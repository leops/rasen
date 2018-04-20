rasen-dsl
================

The `rasen_dsl` crate provides a bunch of utility function to write shaders as perfectly valid Rust code:
```rust
extern crate rasen;
extern crate rasen_dsl;

use rasen_dsl::prelude::*;

fn main() {
    let shader = Module::new();

    let normal: Value<Vec3> = normalize(shader.input(0, "a_normal"));
    let light = vec3(0.3, -0.5, 0.2);
    let color = vec4(0.25, 0.625, 1.0, 1.0);

    let res = clamp(dot(normal, light), 0.1f32, 1.0f32) * color;
    shader.output(0, "o_color", res);

    let bytecode = shader.build(ShaderType::Fragment).unwrap();
    // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
}
```

This crate is even more experimental than the Rasen compiler itself but it already provides all the features exposed by
the compiler.

Ultimately, the goal for the DSL crate (beside being a statically-checked equivalent of the graph builder) is to expose
an API to test the execution of a shader on the CPU, with all the debugging tools that such an environment provides. The
library currently provides all the conversion primitives to turn your scalar / vectors / matrices into Value<_> types to
test your program, however most GLSL operations are left unimplemented.
