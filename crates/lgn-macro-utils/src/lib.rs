//! Legion Macro Utils
//!
//! TODO: write documentation.

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(clippy::missing_errors_doc)]

extern crate proc_macro;

mod attrs;
mod shape;
mod symbol;

use std::{env, path::PathBuf};

pub use attrs::*;
pub use shape::*;
pub use symbol::*;

use cargo_manifest::{DepsSet, Manifest};
use proc_macro::TokenStream;
use quote::quote;

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
    pub fn maybe_get_path(&self, name: &str) -> Option<syn::Path> {
        const LEGION: &str = "legion";
        const LEGION_INTERNAL: &str = "legion_internal";

        let find_in_deps = |deps: &DepsSet| -> Option<syn::Path> {
            let package = if let Some(dep) = deps.get(name) {
                return Some(Self::parse_str(dep.package().unwrap_or(name)));
            } else if let Some(dep) = deps.get(LEGION) {
                dep.package().unwrap_or(LEGION)
            } else if let Some(dep) = deps.get(LEGION_INTERNAL) {
                dep.package().unwrap_or(LEGION_INTERNAL)
            } else {
                return None;
            };

            let mut path = Self::parse_str::<syn::Path>(package);
            if let Some(module) = name.strip_prefix("lgn_") {
                path.segments.push(Self::parse_str(module));
            }
            Some(path)
        };

        let deps = self.manifest.dependencies.as_ref();
        let deps_dev = self.manifest.dev_dependencies.as_ref();

        deps.and_then(find_in_deps)
            .or_else(|| deps_dev.and_then(find_in_deps))
    }
    pub fn get_path(&self, name: &str) -> syn::Path {
        self.maybe_get_path(name)
            .unwrap_or_else(|| Self::parse_str(name))
    }

    pub fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
        syn::parse(path.parse::<TokenStream>().unwrap()).unwrap()
    }
}

/// Derive a label trait
///
/// # Args
///
/// - `input`: The [`syn::DeriveInput`] for struct that is deriving the label trait
/// - `trait_path`: The path [`syn::Path`] to the label trait
#[allow(clippy::needless_pass_by_value)]
pub fn derive_label(input: syn::DeriveInput, trait_path: syn::Path) -> TokenStream {
    let ident = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: syn::token::Where::default(),
        predicates: syn::punctuated::Punctuated::default(),
    });
    where_clause.predicates.push(syn::parse2(quote! { Self: Eq + ::std::fmt::Debug + ::std::hash::Hash + Clone + Send + Sync + 'static }).unwrap());

    (quote! {
        impl #impl_generics #trait_path for #ident #ty_generics #where_clause {
            fn dyn_clone(&self) -> std::boxed::Box<dyn #trait_path> {
                std::boxed::Box::new(std::clone::Clone::clone(self))
            }
        }
    })
    .into()
}
