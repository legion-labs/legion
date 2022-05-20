//! `log_fn` and `span_fn` procedural macros
//!
//! Injects instrumentation into sync and async functions.
//!     async trait functions not supported

// crate-specific lint exceptions:
//#![allow()]

use std::collections::HashSet;

use proc_macro2::{Literal, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    Expr, Ident, ItemFn, Local, Stmt, Token,
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

    let statement = match function.sig.asyncness {
        None => {
            parse_quote! {
                lgn_tracing::span_scope!(_METADATA_FUNC, concat!(module_path!(), "::", #function_name));
            }
        }
        Some(_) => {
            parse_quote! {
                lgn_tracing::async_span_scope!(_METADATA_FUNC, concat!(module_path!(), "::", #function_name));
            }
        }
    };

    if function.sig.asyncness.is_none() {
        function.block.stmts.insert(0, statement);

        return proc_macro::TokenStream::from(quote! {
            #function
        });
    }

    let mut statements = vec![statement];

    let mut index = 0;

    for stmt in function.block.stmts {
        match stmt {
            // Plain .await statement terminated by a ;
            Stmt::Semi(Expr::Await(_), _) => {
                let scope_name =
                    Ident::new(&format!("_METADATA_AWAIT_{}", index), Span::call_site());

                statements.push(parse_quote! {
                    {
                        lgn_tracing::async_span_scope!(
                            #scope_name,
                            concat!(module_path!(), "::", #function_name)
                        );

                        #stmt
                    };
                });

                index += 1;
            }
            // Let binding that depends on an .await
            Stmt::Local(Local {
                ref pat,
                init: Some((_, ref expr)),
                ..
            }) => {
                if let Expr::Await(_) = expr.as_ref() {
                    let scope_name =
                        Ident::new(&format!("_METADATA_AWAIT_{}", index), Span::call_site());

                    statements.push(parse_quote! {
                        let #pat = {
                            lgn_tracing::async_span_scope!(
                                #scope_name,
                                concat!(module_path!(), "::", #function_name)
                            );

                            let result = #expr;

                            result
                        };
                    });

                    index += 1;
                } else {
                    statements.push(stmt);
                }
            }
            _ => statements.push(stmt),
        };
    }

    function.block.stmts = statements;

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
