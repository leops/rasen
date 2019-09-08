extern crate rasen_dsl;

use rasen_dsl::prelude::*;

include!("../../tests/dsl.rs");

#[test]
fn test_build_basic_vert() {
    let _graph = Module::build(|module| {
        let (pos, norm, uv) = basic_vert(
            module.input(0, "a_pos"),
            module.input(1, "a_normal"),
            module.input(2, "a_uv"),
            module.uniform(0, "projection"),
            module.uniform(1, "view"),
            module.uniform(2, "model"),
        );

        module.output(0, "v_pos", pos);
        module.output(1, "v_norm", norm);
        module.output(2, "a_uv", uv);
    });
}
