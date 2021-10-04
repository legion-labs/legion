//! Tauri plugin for Legion's ECS.
//!
//! Provides Tauri integration into Legion's ECS.
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

use legion_app::prelude::*;

use std::sync::{Arc, Mutex};

pub use legion_tauri_macros::*;

pub type TauriPluginAppBuilderFn<R> = Box<dyn FnOnce() -> tauri::App<R> + Send + 'static>;

/// Provides game-engine integration into Tauri's event loop.
pub struct TauriPlugin<R: tauri::Runtime> {
    builder_fn: Arc<Mutex<Option<TauriPluginAppBuilderFn<R>>>>,
}

impl<R: tauri::Runtime> TauriPlugin<R> {
    /// Create a new Tauri Plugin instance from an existing `tauri::Builder<R>`.
    pub fn new_from_builder<A: tauri::Assets>(
        builder: tauri::Builder<R>,
        context: tauri::Context<A>,
    ) -> Self
    where
        tauri::Builder<R>: Send,
    {
        Self::new_from_builder_fn(move || {
            builder
                .build(context)
                .expect("failed to build Tauri application")
        })
    }

    /// Create a new Tauri Plugin instance from a builder function.
    pub fn new_from_builder_fn(
        builder_fn: impl FnOnce() -> tauri::App<R> + Send + 'static,
    ) -> Self {
        Self {
            builder_fn: Arc::new(Mutex::new(Some(Box::new(builder_fn)))),
        }
    }

    fn runner_with(tauri_app: tauri::App<R>, app: App) {
        // FIXME: Once https://github.com/tauri-apps/tauri/pull/2667 is merged, we can
        // get rid of this and move the value directly instead.
        let app = std::rc::Rc::new(std::cell::RefCell::new(app));

        tauri_app.run(move |_, event| {
            if let tauri::Event::MainEventsCleared = event {
                app.borrow_mut().update();
            }
        });
    }
}

impl<R: tauri::Runtime> Plugin for TauriPlugin<R> {
    fn build(&self, app: &mut App) {
        let mut builder_fn = self
            .builder_fn
            .lock()
            .expect("failed to lock the builder function when building the Tauri plugin");

        let builder_fn = std::mem::replace(&mut *builder_fn, None)
            .expect("the builder function was called more than once");

        let tauri_app = builder_fn();
        app.set_runner(move |app| Self::runner_with(tauri_app, app));
    }
}
