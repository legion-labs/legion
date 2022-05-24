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
    Expr, ExprBlock, ExprIf, ExprLoop, ExprWhile, Ident, ItemFn, Local, Stmt, Token,
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

fn decorate_await(stmts: &mut [Stmt]) {
    let mut index = 0;

    while index < stmts.len() {
        match stmts.get_mut(index) {
            // Plain .await statement terminated by a ;
            Some(ref stmt @ Stmt::Semi(Expr::Await(_), _)) => {
                stmts[index] = parse_quote! {
                    {
                        guard_named.end();
                        #stmt
                        guard_named.begin();
                    };
                };
            }

            // Let binding that depends on an .await
            Some(Stmt::Local(Local {
                ref pat,
                init: Some((_, ref expr)),
                ..
            })) => {
                if let Expr::Await(_) = expr.as_ref() {
                    stmts[index] = parse_quote! {
                        let #pat = {
                            guard_named.end();
                            let result = #expr;
                            guard_named.begin();
                            result
                        };
                    };
                }
            }

            Some(
                Stmt::Expr(
                    Expr::Block(ExprBlock { ref mut block, .. })
                    | Expr::Loop(ExprLoop {
                        body: ref mut block,
                        ..
                    })
                    | Expr::While(ExprWhile {
                        body: ref mut block,
                        ..
                    })
                    | Expr::If(ExprIf {
                        then_branch: ref mut block,

                        else_branch: None,
                        ..
                    }),
                )
                | Stmt::Semi(Expr::Block(ExprBlock { ref mut block, .. }), _),
            ) => {
                decorate_await(&mut block.stmts);
            }

            _ => {}
        };

        index += 1;
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
            let mut guard_named = lgn_tracing::guards::ThreadSpanGuard::new(&_METADATA_FUNC);
        },
    );

    decorate_await(&mut function.block.stmts);

    // println!(
    //     "{}",
    //     proc_macro::TokenStream::from(quote! {
    //         #function
    //     })
    //     .to_string()
    // );

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
