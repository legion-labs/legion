//! derive-transit library
//! provides procedural macros like #[derive(Reflect)] to allow reflection and fast serialization
//!

// BEGIN - Legion Labs lints v0.2
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
    broken_intra_doc_links,
    private_intra_doc_links,
    missing_crate_level_docs,
    rust_2018_idioms
)]
// END - Legion Labs standard lints v0.2
// crate-specific exceptions:
#![allow()]

use proc_macro::TokenStream;
use quote::*;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let udt_identifier = ast.ident.clone();
    let udt_name = format!("{}", ast.ident);
    let mut members = Vec::new();

    match ast.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(named_fields) => {
                for field in named_fields.named {
                    let field_name = field.ident.unwrap().to_string();
                    let field_type_name = match field.ty {
                        syn::Type::Array(_) => panic!("Array field type not supported"),
                        syn::Type::BareFn(_) => panic!("BareFn field type not supported"),
                        syn::Type::Group(_) => panic!("Group field type not supported"),
                        syn::Type::ImplTrait(_) => panic!("ImplTrait field type not supported"),
                        syn::Type::Infer(_) => panic!("Infer field type not supported"),
                        syn::Type::Macro(_) => panic!("Macro field type not supported"),
                        syn::Type::Never(_) => panic!("Never field type not supported"),
                        syn::Type::Paren(_) => panic!("Paren field type not supported"),
                        syn::Type::Path(type_path) => {
                            let prefix = match type_path.path.leading_colon {
                                Some(_) => "::",
                                None => "",
                            };
                            let segments: Vec<String> = type_path
                                .path
                                .segments
                                .iter()
                                .map(|s| s.ident.to_string())
                                .collect();
                            String::from(prefix) + &segments.join("::")
                        }
                        syn::Type::Ptr(_) => panic!("Ptr field type not supported"),
                        syn::Type::Reference(_) => panic!("Reference field type not supported"),
                        syn::Type::Slice(_) => panic!("Slice field type not supported"),
                        syn::Type::TraitObject(_) => panic!("TraitObject field type not supported"),
                        syn::Type::Tuple(_) => panic!("Tuple field type not supported"),
                        syn::Type::Verbatim(_) => panic!("Verbatim field type not supported"),
                        unknown_field_type => {
                            panic!("Unexpected field type: {:?}", unknown_field_type)
                        }
                    };
                    members.push((field_name, field_type_name));
                }
            }
            syn::Fields::Unnamed(_) => panic!("only named fields are supported"),
            syn::Fields::Unit => panic!("unit fields not expected"),
        },
        syn::Data::Enum(_) => panic!("enums not supported"),
        syn::Data::Union(_) => panic!("unions not supported"),
    }

    let members_toks = members.iter().map(|m| {
        let member_name = &m.0;
        let member_ident = format_ident!("{}", &m.0);
        let member_type_ident = format_ident!("{}", &m.1);
        quote! {
            Member{ name: #member_name,
                    offset: memoffset::offset_of!(#udt_identifier,#member_ident),
                    size: std::mem::size_of::<#member_type_ident>()},
        }
    });

    TokenStream::from(quote! {
        #[macro_use]
        impl transit::Reflect for MyTestEvent{
            fn reflect() -> UserDefinedType{
                UserDefinedType{
                    name: #udt_name,
                    size: std::mem::size_of::<#udt_identifier>(),
                    members: vec![#(#members_toks)*],
                }
            }
        }
    })
}
