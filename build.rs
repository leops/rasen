extern crate aster;
extern crate syntex_syntax as syntax;

extern crate hyper;
extern crate json;

use std::io::prelude::*;
use std::fs::File;
use hyper::Client;

use aster::*;
use syntax::print::pprust::*;

fn main() {
    let client = Client::new();
    let mut res = client.get(
        "http://www.khronos.org/registry/spir-v/api/1.1/spirv.json"
    ).send().expect("could not get language definitions");

    let mut data = String::new();
    res.read_to_string(&mut data).expect("could not read response");

    let spirv = json::parse(&data).expect("could not parse input data");

    let code = spirv["spv"]["enum"].members()
        .filter_map(|enum_| {
            if enum_["Name"] == "Op" {
                return None;
            }

            let builder = AstBuilder::new();

            let name = enum_["Name"].as_str().expect("name is not a string");

            let mut entries: Vec<_> = enum_["Values"].entries().collect();
            entries.sort_by(|&(_, a), &(_, b)| a.as_u32().unwrap().cmp(&b.as_u32().unwrap()));

            let mut result = String::new();
            if enum_["Type"] == "Value" {
                let enum_ = builder.item()
                    .attr()
                        .allow(vec!["non_camel_case_types"])
                    .attr()
                        .list("derive")
                            .word("Debug")
                            .word("Copy")
                            .word("Clone")
                            .build()
                    .pub_().enum_(name)
                        .ids(
                            entries.iter()
                                .map(|&(name, _)| name)
                        )
                        .build();

                let impl_ = builder.item()
                    .impl_()
                        .trait_()
                            .id("Into<u32>")
                            .build()
                        .item("into")
                            .method().fn_decl()
                                .self_().value()
                                .return_().u32()
                            .block()
                                .expr().match_().self_()
                                    .with_arms(
                                        entries.iter()
                                            .map(|&(entry, val)|
                                                builder.arm().pat().path().id(name).id(entry).build()
                                                    .body().u32(val.as_u32().unwrap())
                                            )
                                    )
                            .build()
                    .build_ty(builder.ty().id(name));

                result = result + &item_to_string(&enum_) + "\n" + &item_to_string(&impl_) + "\n";
            } else {
                let struct_ = builder.item()
                    .attr()
                        .allow(vec!["non_snake_case"])
                    .attr()
                        .list("derive")
                            .word("Debug")
                            .word("Copy")
                            .word("Clone")
                            .word("Default")
                            .build()
                    .pub_().struct_(name)
                        .with_fields(
                            entries.iter()
                                .map(|&(name, _)|
                                    builder.struct_field(name)
                                        .pub_().ty().bool()
                                )
                        )
                        .build();

                let impl_ = builder.item()
                    .impl_()
                        .trait_()
                            .id("Into<u32>")
                            .build()
                        .item("into")
                            .method().fn_decl()
                                .self_().value()
                                .return_().u32()
                            .block()
                                .stmt().let_().mut_id("value").expr().u32(0)
                                .with_stmts(
                                    entries.iter()
                                        .map(|&(name, value)|
                                            builder.stmt()
                                            .expr().if_().field(name).self_()
                                                .then().stmt()
                                                    .expr().bit_or_assign().id("value").u32(1 << value.as_u32().unwrap())
                                                .build().build()
                                        )
                                )
                                .expr().id("value")
                    .build_ty(builder.ty().id(name));

                result = result + &item_to_string(&struct_) + "\n" + &item_to_string(&impl_) + "\n";
            };

            Some(result)
        })
        .fold(String::new(), |acc, itm| acc + &itm + "\n");

    let mut out = File::create(
        concat!(env!("OUT_DIR"), "/spirv.rs")
    ).expect("could not create output file");
    out.write_fmt(format_args!("{}", code)).expect("could not write output to file");
}
