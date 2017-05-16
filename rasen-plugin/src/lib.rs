//! Compiler plugin providing various syntax extensions to simplify writing shaders with the Rasen DSL
//!
//! # Shader wrapper
//! The `shader` attribute on a function will generate a wrapper function with the `_shader` suffix,
//! returning the corresponding Shader object.
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
//! fn swizzle(a_pos: Value<Vec3>, a_color: Value<Vec4>) -> (Value<Vec2>, Value<Vec3>) {
//!     let pos = idx!(a_pos, xy);
//!     let col = idx!(a_color, rgb);
//!     (pos, col)
//! }
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
//! fn constructors(a_xy: Value<Vec2>, a_zw: Value<Vec2>) -> (Value<Vec3>, Value<Vec4>) {
//!     let pos = vec3!(a_xy, 1.0f32);
//!     let norm = vec4!(a_xy, a_zw);
//!     (pos, norm)
//! }
//! ```

#![feature(plugin_registrar, rustc_private, custom_attribute, box_syntax, i128_type)]

extern crate rustc_plugin;
extern crate syntax;
#[macro_use] extern crate quote;

use syntax::codemap::Span;
use syntax::symbol::Symbol;
use syntax::ext::build::AstBuilder;
use rustc_plugin::registry::Registry;
use syntax::parse::token::{Token, Lit};
use syntax::ext::quote::rt::ExtParseUtils;
use syntax::tokenstream::{TokenStream, TokenTree};
use syntax::ast::{self, Item, ItemKind, MetaItem, FnDecl, FunctionRetTy, TyKind, PatKind, ExprKind};
use syntax::ext::base::{Annotatable, ExtCtxt, SyntaxExtension, MacResult, MacEager, TTMacroExpander};

use quote::Ident;

fn insert_shader_wrapper(ecx: &mut ExtCtxt, _span: Span, _meta_item: &MetaItem, item: Annotatable) -> Vec<Annotatable> {
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

fn idx_macro<'cx>(ecx: &'cx mut ExtCtxt, span: Span, tt: &[TokenTree]) -> Box<MacResult + 'cx> {
    match (&tt[0], &tt[2]) {
        (&TokenTree::Token(_, Token::Ident(obj)), &TokenTree::Token(_, Token::Ident(index))) => {
            let index = format!("{}", index);

            let fields: Vec<_> = {
                index.chars()
                    .map(|field| match field {
                        'x' | 'r' | 's' => 0,
                        'y' | 'g' | 't' => 1,
                        'z' | 'b' | 'p' => 2,
                        'w' | 'a' | 'q' => 3,
                        _ => panic!("invalid composite field {}", field),
                    })
                    .collect()
            };

            let count = fields.len();
            MacEager::expr(
                if count > 1 {
                    ecx.expr_call_ident(
                        span,
                        ast::Ident::from_str(&format!("vec{}", count)),
                        vec![
                            ecx.expr_ident(span, obj),
                            ecx.expr_u32(span, fields[0]),
                        ],
                    )
                } else {
                    ecx.expr_call_ident(
                        span,
                        ast::Ident::from_str("index"),
                        vec![
                            ecx.expr_addr_of(
                                span,
                                ecx.expr_ident(span, obj),
                            ),
                            ecx.expr_u32(span, fields[0]),
                        ],
                    )
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
    fn expand<'cx>(&self, ecx: &'cx mut ExtCtxt, span: Span, ts: TokenStream) -> Box<MacResult + 'cx> {
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
                .filter_map(|tt| match tt {
                    TokenTree::Token(_, token) => match token {
                        Token::Ident(id) => Some(
                            ecx.expr_ident(span, id)
                        ),
                        Token::Literal(lit, _) => match lit {
                            Lit::Integer(name) => {
                                let val: u128 = format!("{}", name).parse().unwrap();
                                Some(
                                    ecx.expr_lit(span, ast::LitKind::Int(
                                        val, self.int_ty.unwrap(),
                                    ))
                                )
                            },
                            Lit::Float(name) => Some(
                                ecx.expr_lit(span, ast::LitKind::Float(
                                    name, self.float_ty.unwrap(),
                                )),
                            ),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                })
                .map(|expr| {
                    ecx.stmt_expr(
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
                    )
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

    for cmp_macro in COMPOSITE_MACROS.into_iter() {
        reg.register_syntax_extension(
            Symbol::intern(cmp_macro.func),
            SyntaxExtension::NormalTT(
                box cmp_macro.clone(),
                None, false,
            ),
        );
    }

    reg.register_syntax_extension(
        Symbol::intern("shader"),
        SyntaxExtension::MultiModifier(box insert_shader_wrapper)
    );
}
