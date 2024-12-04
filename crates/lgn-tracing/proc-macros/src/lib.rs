//! `log_fn` and `span_fn` procedural macros
//!
//! Injects instrumentation into sync and async functions.
//!     async trait functions not supported

// crate-specific lint exceptions:
//#![allow()]

use std::collections::HashSet;

use proc_macro2::Literal;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    visit_mut::VisitMut,
    ExprAwait, Ident, ItemFn, Token,
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
    fn visit_expr_await_mut(&mut self, expr: &mut ExprAwait) {
        // TODO: Use attrs
        let ExprAwait { attrs: _, base, .. } = expr;

        let text = base.to_token_stream().to_string();

        *expr = parse_quote! {
            {
                lgn_tracing::span!(_AWAIT, #text);

                lgn_tracing::spans::Instrumentation::new(
                    #base,
                    &_AWAIT,
                    &__lgn_tracing_is_idle
                )
            }.await
        };
    }
}

#[proc_macro_attribute]
pub fn span_fn(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut function = parse_macro_input!(input as ItemFn);

    if function.sig.asyncness.is_some() {
        // NOOP For now
        return proc_macro::TokenStream::from(quote! {
            #function
        });
    };

    let args = parse_macro_input!(args as TraceArgs);

    let function_name = args
        .alternative_name
        .map_or(function.sig.ident.to_string(), |n| n.to_string());

    AwaitVisitor.visit_block_mut(&mut function.block);

    if let Some((last_stmt, stmts)) = function.block.stmts.split_last() {
        function.block.stmts = vec![
            parse_quote! {
                let __lgn_tracing_output = {
                    lgn_tracing::span!(_METADATA_FUNC, concat!(module_path!(), "::", #function_name));
                    let __lgn_tracing_is_idle = std::sync::atomic::AtomicBool::new(false);
                    lgn_tracing::dispatch::on_begin_scope(&_METADATA_FUNC);

                    #(#stmts)*

                    let __lgn_tracing_output = { #last_stmt };
                    lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);
                    __lgn_tracing_output
                };
            },
            parse_quote! {
                return __lgn_tracing_output;
            },
        ];
    } else {
        function.block.stmts = vec![
            parse_quote! {
                let __lgn_tracing_output = {
                    lgn_tracing::span!(_METADATA_FUNC, concat!(module_path!(), "::", #function_name));
                    let __lgn_tracing_is_idle = std::sync::atomic::AtomicBool::new(false);
                    lgn_tracing::dispatch::on_begin_scope(&_METADATA_FUNC);
                    lgn_tracing::dispatch::on_end_scope(&_METADATA_FUNC);
                };
            },
            parse_quote! {
                return __lgn_tracing_output;
            },
        ];
    }

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
