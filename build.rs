#![feature(inclusive_range_syntax)]
#![cfg_attr(feature = "nightly", feature(rustc_private))]

extern crate aster;

#[cfg(feature = "nightly")]
extern crate syntax;

#[cfg(not(feature = "nightly"))]
extern crate syntex_syntax as syntax;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const INTS: [(&'static str, &'static str, &'static str); 3] = [
    ("Bool", "b", "bool"),
    ("Int", "i", "i32"),
    ("UInt", "u", "u32"),
];
const FLOATS: [(&'static str, &'static str, &'static str); 2] = [
    ("Float", "", "f32"),
    ("Double", "d", "f64"),
];

fn main() {
    let builder = aster::AstBuilder::new();

    let mut const_types = Vec::new();
    let mut from_string_arms = Vec::new();
    let mut typed_variants = Vec::new();
    let mut type_name_arms = Vec::new();
    let mut register_constant_arms = Vec::new();

    for &(name, _, ty) in INTS.iter().chain(FLOATS.iter()) {
        from_string_arms.push(
            builder.arm()
                .pat().expr().str(&name.to_string().to_lowercase() as &str)
                .body().ok().path().id("TypeName").id(&name.to_string().to_uppercase()).build()
        );

        typed_variants.push(
            builder.variant(name)
                .tuple()
                    .ty().id(ty)
                    .build()
        );

        type_name_arms.push(
            builder.arm()
                .pat().struct_()
                    .global().id("TypedValue").id(name).build().etc()
                .body().deref().path().id("TypeName").id(&name.to_string().to_uppercase()).build()
        );

        register_constant_arms.push(
            builder.arm()
                .pat().enum_().id("TypedValue").id(name).build()
                    .id("val")
                .build()
                .body().block()
                    .stmt().let_id("res_type").expr()
                        .method_call("register_type").id("$module")
                            .arg().method_call("to_type_name").id("$constant").build()
                        .build()
                    .stmt().let_id("res_id").expr()
                        .method_call("get_id").id("$module").build()
                    .with_stmt(if ty == "bool" {
                        builder.stmt().expr().if_().id("val")
                            .then().expr().method_call("push").field("declarations").id("$module")
                                .arg().struct_().id("Instruction").id("ConstantTrue").build()
                                    .field("result_type").call().id("TypeId")
                                        .arg().id("res_type")
                                    .build()
                                    .field("result_id").call().id("ResultId")
                                        .arg().id("res_id")
                                    .build()
                                .build()
                            .build()
                            .else_().expr().method_call("push").field("declarations").id("$module")
                                .arg().struct_().id("Instruction").id("ConstantFalse").build()
                                    .field("result_type").call().id("TypeId")
                                        .arg().id("res_type")
                                    .build()
                                    .field("result_id").call().id("ResultId")
                                        .arg().id("res_id")
                                    .build()
                                .build()
                            .build()
                    } else {
                        builder.stmt().expr().block().unsafe_()
                            .stmt().build_let(
                                builder.pat().id("transmuted"),
                                Some(builder.ty().array(if ty == "f64" {2} else {1}).u32()),
                                Some(
                                    builder.expr().call().path().global().id("std").id("mem").id("transmute").build()
                                        .arg().id("val")
                                    .build()
                                ),
                                vec![]
                            )
                            .stmt().expr().method_call("push").field("declarations").id("$module")
                                .arg().struct_().id("Instruction").id("Constant").build()
                                    .field("result_type").call().id("TypeId")
                                        .arg().id("res_type")
                                    .build()
                                    .field("result_id").call().id("ResultId")
                                        .arg().id("res_id")
                                    .build()
                                    .field("val").call().path().id("Box").id("new").build()
                                        .arg().id("transmuted")
                                    .build()
                                .build()
                            .build()
                        .build()
                    })
                    .expr().ok().id("res_id")
        );
    }

    for size in 2...4 {
        for &(name, prefix, ty) in INTS.iter().chain(FLOATS.iter()) {
            let type_variant = format!("{}Vec{}", prefix.to_string().to_uppercase(), size);
            let const_name = type_variant.to_uppercase();

            const_types.push(
                builder.impl_item(const_name.clone()).pub_().const_()
                    .expr().ref_().call().path().id("TypeName").id("Vec").build()
                        .arg().u32(size)
                        .arg().path().id("TypeName").id(name.to_string().to_uppercase()).build()
                    .build()
                    .ty().ref_().lifetime("'static").ty().id("TypeName")
            );

            from_string_arms.push(
                builder.arm()
                    .pat().expr().str(&type_variant.to_lowercase() as &str)
                    .body().ok().path().id("TypeName").id(const_name.clone()).build()
            );

            typed_variants.push(
                builder.variant(type_variant.clone())
                    .tuple()
                        .ty().id(ty)
                        .with_fields(
                            (1..size).map(|_| builder.tuple_field().ty().id(ty))
                        )
                    .build()
            );

            type_name_arms.push(
                builder.arm()
                    .pat().struct_()
                        .global().id("TypedValue").id(type_variant.clone()).build().etc()
                    .body().deref().path().id("TypeName").id(const_name).build()
            );

            register_constant_arms.push(
                builder.arm()
                    .pat().enum_().id("TypedValue").id(type_variant).build()
                        .with_ids(
                            (0..size).map(|i| builder.id(format!("f{}", i)))
                        )
                    .build()
                    .body().block()
                        .with_stmts(
                            (0..size).map(|i| builder.stmt()
                                .let_id(format!("f{}_id", i))
                                .expr().call().id("ValueId")
                                    .arg().try().method_call("register_constant").id("$module")
                                        .arg().call().path().id("TypedValue").id(name).build()
                                            .arg().id(format!("f{}", i))
                                        .build()
                                    .build()
                                .build()
                            )
                        )
                        .stmt().let_id("res_type").expr()
                            .method_call("register_type").id("$module")
                                .arg().method_call("to_type_name").id("$constant").build()
                            .build()
                        .stmt().let_id("res_id").expr()
                            .method_call("get_id").id("$module").build()
                        .stmt().expr().method_call("push").field("declarations").id("$module")
                            .arg().struct_().id("Instruction").id("ConstantComposite").build()
                                .field("result_type").call().id("TypeId")
                                    .arg().id("res_type")
                                .build()
                                .field("result_id").call().id("ResultId")
                                    .arg().id("res_id")
                                .build()
                                .field("flds").call().path().id("Box").id("new").build()
                                    .arg().slice()
                                        .with_exprs(
                                            (0..size).map(|i| builder.expr().id(
                                                format!("f{}_id", i)
                                            ))
                                        )
                                    .build()
                                .build()
                            .build()
                        .build()
                        .expr().ok().id("res_id")
            );
        }

        for &(name, prefix, ty) in FLOATS.iter() {
            let type_variant = format!("{}Mat{}", prefix.to_string().to_uppercase(), size);
            let const_name = type_variant.to_uppercase();

            const_types.push(
                builder.impl_item(const_name.clone()).pub_().const_()
                    .expr().ref_().call().path().id("TypeName").id("Mat").build()
                        .arg().u32(size)
                        .arg().path().id("TypeName").id(name.to_string().to_uppercase()).build()
                    .build()
                    .ty().ref_().lifetime("'static").ty().id("TypeName")
            );

            from_string_arms.push(
                builder.arm()
                    .pat().expr().str(&type_variant.to_lowercase() as &str)
                    .body().ok().path().id("TypeName").id(const_name.clone()).build()
            );

            typed_variants.push(
                builder.variant(type_variant.clone())
                    .tuple()
                        .ty().array((size * size) as usize).id(ty)
                    .build()
            );

            type_name_arms.push(
                builder.arm()
                    .pat().struct_()
                    .global().id("TypedValue").id(type_variant.clone()).build().etc()
                .body().deref().path().id("TypeName").id(const_name).build()
            );
        }
    }

    from_string_arms.push(
        builder.arm()
            .pat().wild()
            .body().err().str("Unknown type name")
    );

    register_constant_arms.push(
        builder.arm()
            .pat().wild()
            .body().err().str("Unsupported constant type")
    );

    let type_name =
        builder.item()
            .attr().list("derive")
                .word("Debug")
                .word("Copy")
                .word("Clone")
                .word("Eq")
                .word("PartialEq")
                .word("Hash")
            .build()
            .pub_().enum_("TypeName")
                .id("Bool")
                .tuple("Int")
                    .ty().bool()
                    .build()
                .tuple("Float")
                    .ty().bool()
                    .build()
                .tuple("Vec")
                    .ty().u32()
                    .ty().ref_().lifetime("'static").ty().id("TypeName")
                    .build()
                .tuple("Mat")
                    .ty().u32()
                    .ty().ref_().lifetime("'static").ty().id("TypeName")
                    .build()
                .build();

    let type_name_impl =
        builder.item().impl_()

            .item("BOOL").pub_().const_()
                .expr().ref_().path().id("TypeName").id("Bool").build()
                .ty().ref_().lifetime("'static").ty().id("TypeName")

            .item("INT").pub_().const_()
                .expr().ref_().call().path().id("TypeName").id("Int").build()
                    .arg().bool(true)
                .build()
                .ty().ref_().lifetime("'static").ty().id("TypeName")

            .item("UINT").pub_().const_()
                .expr().ref_().call().path().id("TypeName").id("Int").build()
                    .arg().bool(false)
                .build()
                .ty().ref_().lifetime("'static").ty().id("TypeName")

            .item("FLOAT").pub_().const_()
                .expr().ref_().call().path().id("TypeName").id("Float").build()
                    .arg().bool(false)
                .build()
                .ty().ref_().lifetime("'static").ty().id("TypeName")

            .item("DOUBLE").pub_().const_()
                .expr().ref_().call().path().id("TypeName").id("Float").build()
                    .arg().bool(true)
                .build()
                .ty().ref_().lifetime("'static").ty().id("TypeName")

            .with_items(const_types)

            .item("from_string").pub_().method().fn_decl()
                .arg().id("ty").ty().ref_().ty().id("str")
                .return_().result()
                    .ref_().lifetime("'static").ty().id("TypeName")
                    .ref_().ty().id("str")
            .block()
                .expr().match_()
                    .id("ty")
                    .with_arms(from_string_arms)
            .build()

            .item("is_bool").pub_().method().fn_decl()
                .self_().ref_()
                .return_().bool()
            .block()
                .expr().eq()
                    .deref().self_()
                    .path().id("TypeName").id("Bool").build()

            .item("is_integer").pub_().method().fn_decl()
                .self_().ref_()
                .return_().bool()
            .block()
                .expr().or()
                    .eq()
                        .deref().self_()
                        .deref().path().id("TypeName").id("INT").build()
                    .eq()
                        .deref().self_()
                        .deref().path().id("TypeName").id("UINT").build()

            .item("is_signed").pub_().method().fn_decl()
                .self_().ref_()
                .return_().bool()
            .block()
                .expr().eq()
                    .deref().self_()
                    .deref().path().id("TypeName").id("INT").build()

            .item("is_float").pub_().method().fn_decl()
                .self_().ref_()
                .return_().bool()
            .block()
                .expr().or()
                    .eq()
                        .deref().self_()
                        .deref().path().id("TypeName").id("FLOAT").build()
                    .eq()
                        .deref().self_()
                        .deref().path().id("TypeName").id("DOUBLE").build()

            .item("is_num").pub_().method().fn_decl()
                .self_().ref_()
                .return_().bool()
            .block()
                .expr().or()
                    .method_call("is_integer").self_().build()
                    .method_call("is_float").self_().build()

            .item("is_scalar").pub_().method().fn_decl()
                .self_().ref_()
                .return_().bool()
            .block()
                .expr().or()
                    .method_call("is_bool").self_().build()
                    .method_call("is_num").self_().build()

        .ty().id("TypeName");

    let typed_value =
        builder.item()
            .attr().list("derive")
                .word("Debug")
                .word("Copy")
                .word("Clone")
            .build()
            .pub_().enum_("TypedValue")
                .with_variants(typed_variants)
                .build();

    let typed_value_impl =
        builder.item().impl_()

            .item("to_type_name").pub_().method().fn_decl()
                .self_().ref_()
                .return_().id("TypeName")
            .block()
                .expr().match_()
                    .deref().self_()
                    .with_arms(type_name_arms)
            .build()

        .ty().id("TypedValue");

    let out_dir = env::var("OUT_DIR").unwrap();
    let path_types = Path::new(&out_dir).join("types.rs");
    let mut f_types = File::create(&path_types).unwrap();

    write!(f_types, "{}\n{}\n{}\n{}",
        syntax::print::pprust::item_to_string(&type_name),
        syntax::print::pprust::item_to_string(&type_name_impl),
        syntax::print::pprust::item_to_string(&typed_value),
        syntax::print::pprust::item_to_string(&typed_value_impl)
    ).unwrap();

    let register_constant =
        builder.expr()
            .match_().id("$constant")
                .with_arms(register_constant_arms)
            .build();

    let path_module = Path::new(&out_dir).join("module.rs");
    let mut f_module = File::create(&path_module).unwrap();

    write!(f_module,
        "macro_rules! impl_register_constant {{\n( $module:expr, $constant:expr ) => {{\n{}\n}};\n}}",
        syntax::print::pprust::expr_to_string(&register_constant)
    ).unwrap();
}
