rasen-plugin
================

The `rasen_plugin` crate is a compiler plugin exposing a few utility macro and attributes to make writing
shaders in Rust event easier:
```rust
use rasen_dsl::prelude::*;

#[rasen(module)]
pub fn basic_vert(a_pos: Value<Vec3>, projection: Value<Mat4>, view: Value<Mat4>, model: Value<Mat4>) -> Value<Vec4> {
   let mvp = projection * view * model;
   mvp * vec4!(a_pos, 1.0f32)
}
```
