//! Compiler plugin to automatically wrap a function in a shader object
//!
//! ```
//! #![feature(plugin, custom_attribute)]
//! #![plugin(rasen_plugin)]
//!
//! extern crate rasen;
//! extern crate rasen_dsl;
//! use rasen_dsl::prelude::*;
//!
//! #[shader] // This will create the function basic_frag_shader() -> Shader
//! fn basic_frag(a_normal: Value<Vec3>) -> Value<Vec4> {
//!     let normal = normalize(a_normal);
//!     let light = Vec3(0.3, -0.5, 0.2);
//!     let color = Vec4(0.25, 0.625, 1.0, 1.0);
//!
//!     clamp(dot(normal, light), 0.1f32, 1.0f32) * color
//! }
//! ```

#![feature(plugin_registrar, rustc_private, custom_attribute)]

extern crate rustc_plugin;
extern crate syntax;
#[macro_use] extern crate quote;

use syntax::codemap::Span;
use syntax::symbol::Symbol;
use rustc_plugin::registry::Registry;
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension};
use syntax::ast::{Item, ItemKind, MetaItem, FnDecl, FunctionRetTy, TyKind, PatKind};

use quote::Ident;

pub fn insert_shader_wrapper(ecx: &mut ExtCtxt, _span: Span, _meta_item: &MetaItem, item: Annotatable) -> Vec<Annotatable> {
    let mut result = vec![ item.clone() ];

    if let Annotatable::Item(item) = item {
        let Item { ident, ref node, .. } = *item;
        let fn_name = Ident::from(format!("{}", ident));
        let aux_name = Ident::from(format!("{}_shader", ident));

        if let ItemKind::Fn(ref decl, _, _, _, _, ref _block) = *node {
            let FnDecl { ref inputs, ref output, .. } = **decl;

            let args: Vec<_> = {
                inputs.iter()
                    .map(|arg| match arg.pat.node {
                        PatKind::Ident(_, ident, _) => Ident::from(format!("{}", ident.node)),
                        _ => panic!("unimplemented {:?}", arg.pat.node),
                    })
                    .collect()
            };

            let (attributes, uniforms): (Vec<_>, Vec<_>) = {
                args.clone()
                    .into_iter()
                    .partition(|ident| {
                        let name = format!("{}", ident);
                        name.starts_with("a_")
                    })
            };

            let (output, outputs) = match output {
                &FunctionRetTy::Ty(ref ty) => match ty.node {
                    TyKind::Tup(ref fields) => {
                        let list: Vec<_> = {
                            (0..fields.len())
                                .map(|id| {
                                    Ident::from(format!("out_{}", id))
                                })
                                .collect()
                        };

                        let outputs = list.clone();
                        (quote! { ( #( #list ),* ) }, outputs)
                    },
                    TyKind::Path(_, _) => {
                        let output = Ident::from("output");
                        (quote!{ #output }, vec![ output ])
                    },
                    _ => panic!("unimplemented {:?}", ty.node),
                },
                _ => panic!("unimplemented {:?}", output),
            };

            let attr_id: Vec<_> = (0..attributes.len()).map(|i| i as u32).collect();
            let uni_id: Vec<_> = (0..uniforms.len()).map(|i| i as u32).collect();
            let out_id: Vec<_> = (0..outputs.len()).map(|i| i as u32).collect();

            let tokens = quote! {
                #[allow(dead_code)]
                pub fn #aux_name() -> Shader {
                    let shader = Shader::new();
                    #( let #attributes = shader.input(#attr_id); )*
                    #( let #uniforms = shader.uniform(#uni_id); )*
                    let #output = #fn_name( #( #args ),* );
                    #( shader.output(#out_id, #outputs); )*
                    shader
                }
            };

            result.push(Annotatable::Item(ecx.parse_item(tokens.to_string())));
        }
    }

    result
}

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(
        Symbol::intern("shader"),
        SyntaxExtension::MultiModifier(Box::new(insert_shader_wrapper))
    );
}
