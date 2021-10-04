//! Legion Macro Utils
//!
//! TODO: write documentation.
//!
// BEGIN - Legion Labs lints v0.4
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::char_lit_as_u8,
    clippy::checked_conversions,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::fallible_impl_from,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::from_iter_instead_of_collect,
    clippy::if_let_mutex,
    clippy::implicit_clone,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::large_digit_groups,
    clippy::large_stack_arrays,
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
    clippy::missing_enforced_import_renames,
    clippy::mut_mut,
    clippy::mutex_integer,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::option_option,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::single_match_else,
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
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// END - Legion Labs standard lints v0.4
// crate-specific exceptions:
#![allow()]

extern crate proc_macro;

mod attrs;
mod symbol;

pub use attrs::*;
pub use symbol::*;

use cargo_manifest::{DepsSet, Manifest};
use proc_macro::TokenStream;
use std::{env, path::PathBuf};

pub struct LegionManifest {
    manifest: Manifest,
}

impl Default for LegionManifest {
    fn default() -> Self {
        Self {
            manifest: env::var_os("CARGO_MANIFEST_DIR")
                .map(PathBuf::from)
                .map(|mut path| {
                    path.push("Cargo.toml");
                    Manifest::from_path(path).unwrap()
                })
                .unwrap(),
        }
    }
}

impl LegionManifest {
    pub fn get_path(&self, name: &str) -> syn::Path {
        const LEGION: &str = "legion";
        const LEGION_INTERNAL: &str = "legion_internal";

        let find_in_deps = |deps: &DepsSet| -> Option<syn::Path> {
            let package = if let Some(dep) = deps.get(LEGION) {
                dep.package().unwrap_or(LEGION)
            } else if let Some(dep) = deps.get(LEGION_INTERNAL) {
                dep.package().unwrap_or(LEGION_INTERNAL)
            } else {
                return None;
            };

            let mut path = get_path(package);
            if let Some(module) = name.strip_prefix("legion_") {
                path.segments.push(parse_str(module));
            }
            Some(path)
        };

        let deps = self.manifest.dependencies.as_ref();
        let deps_dev = self.manifest.dev_dependencies.as_ref();

        deps.and_then(find_in_deps)
            .or_else(|| deps_dev.and_then(find_in_deps))
            .unwrap_or_else(|| get_path(name))
    }
}

fn get_path(path: &str) -> syn::Path {
    parse_str(path)
}

fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
    syn::parse(path.parse::<TokenStream>().unwrap()).unwrap()
}
