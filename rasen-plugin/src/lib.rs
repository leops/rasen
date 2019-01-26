//! Compiler plugin providing various syntax extensions to simplify writing shaders with the Rasen DSL
//!
//! # Module wrapper
//! The `rasen(module)` attribute on a function will generate a wrapper function with the `_module` suffix,
//! returning the corresponding Module object.
//! 
//! The `rasen(function)` attribute flags a function to be exported when called from a module builder function.
//! Otherwise, due to the way the compiler works on data-flow graph, any function called in Rust will be inlined
//! in the resulting code.
//!
//! ```
//! #![feature(plugin, custom_attribute)]
//! #![plugin(rasen_plugin)]
//!
//! extern crate rasen;
//! extern crate rasen_dsl;
//! use rasen_dsl::prelude::*; // Some features require the prelude to be imported in scope
//!
//! #[rasen(function)] // This function will be exported to the SPIR-V code
//! fn clamp_light(value: Value<Float>) -> Value<Float> {
//!     clamp(value, 0.1f32, 1.0f32)
//! }
//! 
//! #[rasen(module)] // This will create the function basic_frag_module() -> Module
//! fn basic_frag(a_normal: Value<Vec3>) -> Value<Vec4> {
//!     let normal = normalize(a_normal);
//!     let light = Vec3(0.3, -0.5, 0.2);
//!     let color = Vec4(0.25, 0.625, 1.0, 1.0);
//!
//!     clamp_light(dot(normal, light)) * color
//! }
//! # basic_frag_module()
//! ```
//!
//! # Index macro
//! The `idx!` macro allows indexing vector values with the GLSL swizzle syntax:
//!
//! ```
//! # #![feature(plugin)]
//! # #![plugin(rasen_plugin)]
//! # extern crate rasen;
//! # extern crate rasen_dsl;
//! # use rasen_dsl::prelude::*;
//! # #[rasen(module)]
//! fn swizzle(a_pos: Value<Vec3>, a_color: Value<Vec4>) -> (Value<Vec2>, Value<Vec3>) {
//!     let pos = idx!(a_pos, xy);
//!     let col = idx!(a_color, rgb);
//!     (pos, col)
//! }
//! # swizzle_module()
//! ```
//!
//! # Vector macros
//! Finally, there is a macro counterpart for all the vector constructor functions. However, the
//! macros are variadic, and behave similarly to their GLSL counterparts.
//!
//! ```
//! # #![feature(plugin)]
//! # #![plugin(rasen_plugin)]
//! # extern crate rasen;
//! # extern crate rasen_dsl;
//! # use rasen_dsl::prelude::*;
//! # #[rasen(module)]
//! fn constructors(a_xy: Value<Vec2>, a_zw: Value<Vec2>) -> (Value<Vec3>, Value<Vec4>) {
//!     let pos = vec3!(a_xy, 1.0f32);
//!     let norm = vec4!(a_xy, a_zw);
//!     (pos, norm)
//! }
//! # constructors_module()
//! ```

#![recursion_limit="256"]
#![feature(plugin_registrar, rustc_private, custom_attribute, box_syntax)]
#![warn(clippy::all, clippy::pedantic)]

extern crate rustc_plugin;
extern crate syntax;
#[macro_use] extern crate quote;
extern crate proc_macro2;

use syntax::source_map::{Span, FileName};
use syntax::symbol::Symbol;
use syntax::ext::build::AstBuilder;
use rustc_plugin::registry::Registry;
use syntax::parse::{self, token::{Token, Lit}};
use syntax::tokenstream::{TokenStream, TokenTree};
use syntax::ast::{
    self, Item, ItemKind,
    MetaItem, MetaItemKind, NestedMetaItemKind,
    FnDecl, FunctionRetTy, TyKind, PatKind, ExprKind,
};
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension, MacResult, MacEager, TTMacroExpander};
use syntax::source_map::edition::Edition;

use proc_macro2::{Ident, Span as MacroSpan};

#[allow(clippy::cast_possible_truncation)]
fn insert_module_wrapper(ecx: &mut ExtCtxt, span: Span, item: Annotatable) -> Vec<Annotatable> {
    let tokens = if let Annotatable::Item(ref item) = item {
        let Item { ident, ref node, .. } = **item;
        let fn_name = Ident::new(&format!("{}", ident), MacroSpan::call_site());
        let aux_name = Ident::new(&format!("{}_module", ident), MacroSpan::call_site());

        if let ItemKind::Fn(ref decl, ..) = *node {
            let FnDecl { ref inputs, ref output, .. } = **decl;

            let args: Vec<_> = {
                inputs.iter()
                    .map(|arg| match arg.pat.node {
                        PatKind::Ident(_, ident, _) => Ident::new(&format!("{}", ident.name), MacroSpan::call_site()),
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

            let attr_names: Vec<_> = attributes.iter().map(|id| id.to_string()).collect();
            let uni_names: Vec<_> = uniforms.iter().map(|id| id.to_string()).collect();

            let (output, outputs) = match *output {
                FunctionRetTy::Ty(ref ty) => match ty.node {
                    TyKind::Tup(ref fields) => {
                        let list: Vec<_> = {
                            (0..fields.len())
                                .map(|id| {
                                    Ident::new(&format!("out_{}", id), MacroSpan::call_site())
                                })
                                .collect()
                        };

                        let outputs = list.clone();
                        (quote! { ( #( #list ),* ) }, outputs)
                    },
                    TyKind::Path(_, _) => {
                        let output = Ident::new("output", MacroSpan::call_site());
                        (quote!{ #output }, vec![ output ])
                    },
                    _ => panic!("unimplemented {:?}", ty.node),
                },
                _ => panic!("unimplemented {:?}", output),
            };

            let attr_id: Vec<_> = (0..attributes.len()).map(|i| i as u32).collect();
            let uni_id: Vec<_> = (0..uniforms.len()).map(|i| i as u32).collect();
            let out_id: Vec<_> = (0..outputs.len()).map(|i| i as u32).collect();

            Some((format!("{}_module", ident), quote! {
                #[allow(dead_code)]
                pub fn #aux_name() -> Module {
                    let module = Module::new();
                    #( let #attributes = module.input(#attr_id, #attr_names); )*
                    #( let #uniforms = module.uniform(#uni_id, #uni_names); )*
                    let #output = #fn_name( #( #args ),* );
                    #( module.output(#out_id, None, #outputs); )*
                    module
                }
            }))
        } else {
            None
        }
    } else {
        None
    };

    if let Some((name, tokens)) = tokens {
        let mut parser = parse::new_parser_from_source_str(
            ecx.parse_sess,
            FileName::Custom(name),
            tokens.to_string(),
        );
        vec![
            item,
            Annotatable::Item(parser.parse_item().expect("result").expect("option")),
        ]
    } else {
        ecx.span_fatal(span, "Unsupported item for Rasen module attribute")
    }
}

/*
 * fn name(args: T, ...) -> R {
 *     let func = |args: T, ...| -> R {
 *         // Code
 *     };
 * 
 *     if let Some(module) = args.get_module().or_else(...) {
 *         let func = module.function(func);
 *         func(args, ...)
 *     } else {
 *         func(args, ...)
 *     }
 * }
 */
fn insert_function_wrapper(ecx: &mut ExtCtxt, span: Span, item: Annotatable) -> Vec<Annotatable> {
    if let Annotatable::Item(item) = item {
        let Item { ident, ref attrs, ref node, span, .. } = *item;
        if let ItemKind::Fn(ref decl, header, ref generics, ref block) = *node {
            let FnDecl { ref inputs, ref output, .. } = **decl;
            return vec![
                Annotatable::Item(ecx.item(
                    span, ident,
                    attrs.clone(),
                    ItemKind::Fn(
                        ecx.fn_decl(
                            inputs.clone(),
                            output.clone(),
                        ),
                        header,
                        generics.clone(),
                        ecx.block(block.span, vec![
                            ecx.stmt_let(
                                block.span, false,
                                ecx.ident_of("func"),
                                ecx.lambda_fn_decl(
                                    block.span,
                                    ecx.fn_decl(
                                        inputs.clone(),
                                        output.clone(),
                                    ),
                                    ecx.expr_block(block.clone()),
                                    block.span,
                                ),
                            ),
                            ecx.stmt_expr(ecx.expr(block.span, ExprKind::IfLet(
                                vec![
                                    ecx.pat_some(
                                        block.span,
                                        ecx.pat_ident(block.span, ecx.ident_of("module")),
                                    )
                                ],
                                inputs.iter()
                                    .fold(None, |current, arg| match arg.pat.node {
                                        PatKind::Ident(_, ident, _) => Some({
                                            let id = ecx.expr_ident(ident.span, ident);
                                            let module = ecx.expr_method_call(
                                                ident.span, id,
                                                ecx.ident_of("get_module"),
                                                Vec::new(),
                                            );

                                            if let Some(chain) = current {
                                                ecx.expr_method_call(
                                                    ident.span, chain,
                                                    ecx.ident_of("or_else"),
                                                    vec![
                                                        ecx.lambda0(ident.span, module),
                                                    ],
                                                )
                                            } else {
                                                module
                                            }
                                        }),
                                        _ => ecx.span_fatal(arg.pat.span, "Unsupported destructuring"),
                                    })
                                    .unwrap(),
                                ecx.block(block.span, vec![
                                    ecx.stmt_let(
                                        block.span, false,
                                        ecx.ident_of("func"),
                                        ecx.expr_method_call(
                                            block.span,
                                            ecx.expr_ident(block.span, ecx.ident_of("module")),
                                            ecx.ident_of("function"),
                                            vec![
                                                ecx.expr_ident(block.span, ecx.ident_of("func")),
                                            ],
                                        ),
                                    ),
                                    ecx.stmt_expr(ecx.expr_call_ident(
                                        block.span,
                                        ecx.ident_of("func"),
                                        inputs.iter()
                                            .map(|arg| match arg.pat.node {
                                                PatKind::Ident(_, ident, _) => ecx.expr_ident(ident.span, ident),
                                                _ => ecx.span_fatal(arg.pat.span, "Unsupported destructuring"),
                                            })
                                            .collect(),
                                    )),
                                ]),
                                Some(
                                    ecx.expr_call_ident(
                                        block.span,
                                        ecx.ident_of("func"),
                                        inputs.iter()
                                            .map(|arg| match arg.pat.node {
                                                PatKind::Ident(_, ident, _) => ecx.expr_ident(ident.span, ident),
                                                _ => ecx.span_fatal(arg.pat.span, "Unsupported destructuring"),
                                            })
                                            .collect(),
                                    ),
                                ),
                            ))),
                        ]),
                    ),
                )),
            ];
        }
    }

    ecx.span_fatal(span, "Unsupported item for Rasen function attribute")
}

fn rasen_attribute(ecx: &mut ExtCtxt, _: Span, meta_item: &MetaItem, item: Annotatable) -> Vec<Annotatable> {
    let res = match meta_item.node {
        MetaItemKind::List(ref list) if list.len() == 1 => {
            let first = &list[0];
            match first.node {
                NestedMetaItemKind::MetaItem(MetaItem { ref ident, node: MetaItemKind::Word, span }) => {
                    if ident.segments.len() == 1 {
                        let segment = &ident.segments[0];
                        if segment.ident.name == Symbol::intern("module") {
                            Ok(insert_module_wrapper(ecx, span, item))
                        } else if segment.ident.name == Symbol::intern("function") {
                            Ok(insert_function_wrapper(ecx, span, item))
                        } else {
                            Err(span)
                        }
                    } else {
                        Err(span)
                    }
                },
                _ => Err(first.span)
            }
        },
        _ => Err(meta_item.span),
    };

    match res {
        Ok(res) => res,
        Err(span) => ecx.span_fatal(span, "Unsupported rasen attribute"),
    }
}

fn idx_macro<'cx>(ecx: &'cx mut ExtCtxt, span: Span, tt: &[TokenTree]) -> Box<MacResult + 'cx> {
    match (&tt[0], &tt[2]) {
        (&TokenTree::Token(_, Token::Ident(obj, _)), &TokenTree::Token(_, Token::Ident(index, _))) => {
            let index = index.to_string();
            if index.is_empty() {
                ecx.span_fatal(span, "Empty composite field");
            }

            let mut fields: Vec<_> = {
                index.chars()
                    .map(|field| {
                        let index = match field {
                            'x' | 'r' | 's' => 0,
                            'y' | 'g' | 't' => 1,
                            'z' | 'b' | 'p' => 2,
                            'w' | 'a' | 'q' => 3,
                            _ => ecx.span_fatal(span, &format!("invalid composite field {}", field)),
                        };

                        ecx.expr_call_ident(
                            span,
                            ast::Ident::from_str("index"),
                            vec![
                                ecx.expr_addr_of(
                                    span,
                                    ecx.expr_ident(span, obj),
                                ),
                                ecx.expr_u32(span, index),
                            ],
                        )
                    })
                    .collect()
            };

            let count = fields.len();
            MacEager::expr(
                if count > 1 {
                    ecx.expr_call_ident(
                        span,
                        ast::Ident::from_str(&format!("vec{}", count)),
                        fields,
                    )
                } else {
                    fields.remove(0)
                }
            )
        },
        _ => {
            box MacEager::default()
        },
    }
}

#[derive(Clone)]
struct CompositeMacro<'a>{
    pub func: &'a str,
    pub float_ty: Option<ast::FloatTy>,
    pub int_ty: Option<ast::LitIntType>,
}

impl<'a> TTMacroExpander for CompositeMacro<'a> {
    fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, span: Span, ts: TokenStream, _: Option<Span>) -> Box<MacResult + 'cx> {
        let func = ast::Ident::from_str(self.func);
        let vec = ast::Ident::from_str("vec");

        let size = self.func.chars().last().unwrap().to_digit(10).unwrap() as usize;
        let indices: Vec<_> = {
            (0..size)
                .map(|i| {
                    ecx.expr_method_call(
                        span,
                        ecx.expr(span, ExprKind::Index(
                            ecx.expr_ident(span, vec),
                            ecx.expr_usize(span, i),
                        )),
                        ast::Ident::from_str("clone"),
                        vec![],
                    )
                })
                .collect()
        };

        let mut block = vec![
            ecx.stmt_let(
                span, true, vec,
                ecx.expr_vec_ng(span),
            ),
        ];

        block.extend(
            ts.trees()
                .filter_map(|tt| {
                    let expr = match tt {
                        TokenTree::Token(_, token) => match token {
                            Token::Ident(id, _) => ecx.expr_ident(span, id),
                            Token::Literal(lit, _) => match lit {
                                Lit::Integer(name) => {
                                    let val: u128 = format!("{}", name).parse().unwrap();
                                    ecx.expr_lit(span, ast::LitKind::Int(
                                        val, self.int_ty.unwrap(),
                                    ))
                                },
                                Lit::Float(name) => ecx.expr_lit(span, ast::LitKind::Float(
                                    name, self.float_ty.unwrap(),
                                )),
                                _ => return None,
                            },
                            _ => return None,
                        },
                        _ => return None,
                    };

                    Some(ecx.stmt_expr(
                        ecx.expr_method_call(
                            span,
                            ecx.expr_ident(span, vec),
                            ast::Ident::from_str("extend"),
                            vec![
                                ecx.expr_call(
                                    span,
                                    ecx.expr_path(ecx.path(span, vec![
                                        ast::Ident::from_str("ValueIter"),
                                        ast::Ident::from_str("iter"),
                                    ])),
                                    vec![
                                        ecx.expr_addr_of(span, expr),
                                    ],
                                ),
                            ],
                        ),
                    ))
                })
        );

        block.push(
            ecx.stmt_expr(
                ecx.expr_call_ident(span, func, indices),
            ),
        );

        MacEager::expr(
            ecx.expr_block(
                ecx.block(span, block),
            ),
        )
    }
}

const COMPOSITE_MACROS: &[CompositeMacro<'static>] = &[
    CompositeMacro {
        func: "bvec2",
        float_ty: None,
        int_ty: None,
    },
    CompositeMacro {
        func: "bvec3",
        float_ty: None,
        int_ty: None,
    },
    CompositeMacro {
        func: "bvec4",
        float_ty: None,
        int_ty: None,
    },
    CompositeMacro {
        func: "dvec2",
        float_ty: Some(ast::FloatTy::F64),
        int_ty: None,
    },
    CompositeMacro {
        func: "dvec3",
        float_ty: Some(ast::FloatTy::F64),
        int_ty: None,
    },
    CompositeMacro {
        func: "dvec4",
        float_ty: Some(ast::FloatTy::F64),
        int_ty: None,
    },
    CompositeMacro {
        func: "ivec2",
        float_ty: None,
        int_ty: Some(ast::LitIntType::Signed(ast::IntTy::I32)),
    },
    CompositeMacro {
        func: "ivec3",
        float_ty: None,
        int_ty: Some(ast::LitIntType::Signed(ast::IntTy::I32)),
    },
    CompositeMacro {
        func: "ivec4",
        float_ty: None,
        int_ty: Some(ast::LitIntType::Signed(ast::IntTy::I32)),
    },
    CompositeMacro {
        func: "uvec2",
        float_ty: None,
        int_ty: Some(ast::LitIntType::Unsigned(ast::UintTy::U32)),
    },
    CompositeMacro {
        func: "uvec3",
        float_ty: None,
        int_ty: Some(ast::LitIntType::Unsigned(ast::UintTy::U32)),
    },
    CompositeMacro {
        func: "uvec4",
        float_ty: None,
        int_ty: Some(ast::LitIntType::Unsigned(ast::UintTy::U32)),
    },
    CompositeMacro {
        func: "vec2",
        float_ty: Some(ast::FloatTy::F32),
        int_ty: None,
    },
    CompositeMacro {
        func: "vec3",
        float_ty: Some(ast::FloatTy::F32),
        int_ty: None,
    },
    CompositeMacro {
        func: "vec4",
        float_ty: Some(ast::FloatTy::F32),
        int_ty: None,
    },
];

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("idx", idx_macro);

    for cmp_macro in COMPOSITE_MACROS {
        reg.register_syntax_extension(
            Symbol::intern(cmp_macro.func),
            SyntaxExtension::NormalTT {
                expander: box cmp_macro.clone(),
                allow_internal_unsafe: false,
                allow_internal_unstable: false,
                def_info: None,
                unstable_feature: None,
                local_inner_macros: false,
                edition: Edition::Edition2018,
            },
        );
    }

    reg.register_syntax_extension(
        Symbol::intern("rasen"),
        SyntaxExtension::MultiModifier(box rasen_attribute)
    );
}
