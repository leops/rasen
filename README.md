rasen
================

[![crates.io](https://img.shields.io/crates/v/rasen.svg)](https://crates.io/crates/rasen/)
[![AppVeyor Status](https://ci.appveyor.com/api/projects/status/github/leops/rasen?svg=true)](https://ci.appveyor.com/project/leops/rasen)
[![Travis Status](https://travis-ci.org/leops/rasen.svg?branch=master)](https://travis-ci.org/leops/rasen/)

Generate SPIR-V bytecode from an operation graph (heavy WIP)

```rust
#[macro_use]
extern crate rasen;

use rasen::*;
use rasen::TypedValue::*;

fn main() {
    let graph = rasen_graph! {
        // The only output of this graph is a vec4, at location 0
        Output(0, TypeName::VEC4) {
            // Multiply the light intensity by the surface color
            Multiply {
                // Restrict the intensity into the ambient light range
                Clamp {
                    // Compute the dot product of the surface normal and the light direction
                    Dot {
                        // Normalize the normal
                        Normalize {
                            // The surface normal, a vec3 input at location 0
                            Input(0, TypeName::VEC3)
                        }
                        // The directional light direction
                        Constant(Vec3(0.3, -0.5, 0.2))
                    }
                    // The minimum / maximum light levels
                    Constant(Float(0.1))
                    Constant(Float(1.0))
                }
                // The Material color
                Constant(Vec4(0.25, 0.625, 1.0, 1.0))
            }
        };
    };

    let bytecode = build_program(&graph, ShaderType::Fragment).unwrap();
    // bytecode is now a Vec<u8> you can pass to Vulkan to create the shader module
}
```
