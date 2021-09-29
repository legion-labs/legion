//! Tauri plugin macros for Legion's ECS.
//!
//! Provides Tauri integration into Legion's ECS.
//!
// BEGIN - Legion Labs lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_ok_or,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::todo,
    clippy::trait_duplication_in_bounds,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::useless_transmute,
    clippy::use_self,
    clippy::verbose_file_reads,
    clippy::zero_sized_map_values,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links
)]
// END - Legion Labs standard lints v0.3
// crate-specific exceptions:
#![allow()]

//extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_quote, FnArg, Ident, ItemFn};

#[proc_macro_attribute]
pub fn legion_tauri_command(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    assert!(args.is_empty());

    let function = parse_macro_input!(input as ItemFn);
    proc_macro::TokenStream::from(legion_tauri_command_impl(function))
}

fn legion_tauri_command_impl(mut function: syn::ItemFn) -> TokenStream {
    match get_raw_return_type(&function) {
        Some(raw_return_type) => {
            // Let's create an exposed function that grabs the original name and calls
            // the original function.
            let name = format!("__{}_impl", function.sig.ident);
            let name = Ident::new(&name, Span::call_site());

            let args = get_arguments(&function);
            let mut exposed_function = function.clone();
            exposed_function.sig.output = syn::ReturnType::Type(
                syn::token::RArrow::default(),
                Box::new(to_tauri_result_type(&raw_return_type)),
            );
            exposed_function.block.stmts = vec![parse_quote! {
                return match #name(#(#args),*).await {
                    Ok(v) => Ok(v),
                    Err(e) => Err(format!("{}", e)),
                };
            }];

            function.sig.ident = name;

            let result = quote! {
                #function

                #[tauri::command]
                #exposed_function
            };

            result
        }
        None => quote! {
            #[tauri::command]
            #function
        },
    }
}

fn get_raw_return_type(function: &syn::ItemFn) -> Option<syn::Type> {
    if let syn::ReturnType::Type(_, t) = &function.sig.output {
        if let syn::Type::Path(p) = t.as_ref() {
            for segment in &p.path.segments {
                if segment.ident.to_string().eq("Result") {
                    if let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments {
                        for argument in &arguments.args {
                            if let syn::GenericArgument::Type(t) = &argument {
                                return Some(t.clone());
                            }
                        }
                    }
                }
            }
        };
    }

    None
}

fn get_arguments(function: &syn::ItemFn) -> Vec<TokenStream> {
    function
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                syn::Pat::Ident(arg) => quote! {#arg},
                pat => {
                    panic!("unsupported argument type: {:?}", pat);
                }
            },
            FnArg::Receiver(_) => {
                panic!("unable to use self as a command function parameter");
            }
        })
        .collect()
}

fn to_tauri_result_type(t: &syn::Type) -> syn::Type {
    parse_quote! {Result<#t, String>}
}
