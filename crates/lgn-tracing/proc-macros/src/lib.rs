//! `log_fn` and `span_fn` procedural macros
//!
//! Injects instrumentation into sync and async functions.
//!     async trait functions not supported

// crate-specific lint exceptions:
//#![allow()]

use std::collections::HashSet;

use proc_macro2::Literal;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    Expr, ExprReturn, Ident, ItemFn, Local, Stmt, Token,
};

struct TraceArgs {
    alternative_name: Option<Literal>,
}

impl Parse for TraceArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            Ok(Self {
                alternative_name: None,
            })
        } else {
            Ok(Self {
                alternative_name: Some(Literal::parse(input)?),
            })
        }
    }
}

struct AwaitVisitor;

impl VisitMut for AwaitVisitor {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        if let Stmt::Semi(Expr::Await(_), _) = stmt {
            *stmt = parse_quote! {
                {
                    lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);
                    #stmt
                    lgn_tracing::dispatch::on_begin_scope(&_METADATA_FUNC);
                };
            };

            return;
        }

        if let Stmt::Local(Local {
            ref pat,
            init: Some((_, ref expr)),
            ..
        }) = stmt
        {
            if let Expr::Await(_) = expr.as_ref() {
                *stmt = parse_quote! {
                    let #pat = {
                        lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);
                        let result = #expr;
                        lgn_tracing::dispatch::on_begin_scope(&_METADATA_FUNC);
                        result
                    };
                };

                return;
            }
        }

        visit_mut::visit_stmt_mut(self, stmt);
    }
}

#[proc_macro_attribute]
pub fn span_fn(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = parse_macro_input!(args as TraceArgs);
    let mut function = parse_macro_input!(input as ItemFn);

    let function_name = args
        .alternative_name
        .map_or(function.sig.ident.to_string(), |n| n.to_string());

    if function.sig.asyncness.is_none() {
        function.block.stmts.insert(0, parse_quote! {
            lgn_tracing::span_scope!(_METADATA_FUNC, concat!(module_path!(), "::", #function_name));
        });

        return proc_macro::TokenStream::from(quote! {
            #function
        });
    }

    function.block.stmts.insert(0, parse_quote! {
        static _METADATA_FUNC: lgn_tracing::spans::SpanMetadata = lgn_tracing::spans::SpanMetadata {
            name: concat!(module_path!(), "::", #function_name),
            location: lgn_tracing::spans::SpanLocation {
                lod: lgn_tracing::Verbosity::Max,
                target: module_path!(),
                module_path: module_path!(),
                file: file!(),
                line: line!()
            }
        };
    });

    function.block.stmts.insert(
        1,
        parse_quote! {
            lgn_tracing::dispatch::on_begin_scope(&_METADATA_FUNC);
        },
    );

    AwaitVisitor.visit_block_mut(&mut function.block);

    if let Some(last_stmt) = function.block.stmts.last_mut() {
        if let Stmt::Semi(Expr::Return(ExprReturn { attrs: _, expr, .. }), _) = last_stmt {
            // TODO: Handle attrs?
            if let Some(expr) = expr {
                *last_stmt = parse_quote! {
                    {
                        let __lgn_tracing_returned_value = #expr;

                        lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);

                        return __lgn_tracing_returned_value;
                    };
                }
            } else {
                *last_stmt = parse_quote! {
                    lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);
                }
            }
        } else {
            *last_stmt = parse_quote! {
                {
                    let __lgn_tracing_returned_value = #last_stmt;

                    lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);

                    return __lgn_tracing_returned_value;
                };
            }
        };
    };

    proc_macro::TokenStream::from(quote! {
        #function
    })
}

struct LogArgs {
    #[allow(unused)]
    vars: HashSet<Ident>,
}

impl Parse for LogArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let vars = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Self {
            vars: vars.into_iter().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn log_fn(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    assert!(args.is_empty());
    let mut function = parse_macro_input!(input as ItemFn);
    let function_name = function.sig.ident.to_string();

    function.block.stmts.insert(
        0,
        parse_quote! {
            lgn_tracing::trace!(#function_name);
        },
    );
    proc_macro::TokenStream::from(quote! {
        #function
    })
}
