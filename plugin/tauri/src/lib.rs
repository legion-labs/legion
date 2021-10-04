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

use std::marker::PhantomData;

use legion_app::prelude::*;

pub use legion_tauri_macros::*;

pub struct TauriPluginSettings<R: tauri::Runtime, A: tauri::Assets> {
    builder: tauri::Builder<R>,
    context: tauri::Context<A>,
}

impl<R: tauri::Runtime, A: tauri::Assets> TauriPluginSettings<R, A> {
    pub fn new(builder: tauri::Builder<R>, context: tauri::Context<A>) -> Self {
        Self { builder, context }
    }

    pub fn new_with_plugin(
        builder: tauri::Builder<R>,
        context: tauri::Context<A>,
    ) -> (Self, TauriPlugin<R, A>) {
        (Self { builder, context }, Self::plugin())
    }

    fn plugin() -> TauriPlugin<R, A> {
        TauriPlugin::default()
    }
}

/// Provides game-engine integration into Tauri's event loop.
pub struct TauriPlugin<R: tauri::Runtime, A: tauri::Assets> {
    phantom: PhantomData<fn() -> (R, A)>,
}

impl<R: tauri::Runtime, A: tauri::Assets> Default for TauriPlugin<R, A> {
    fn default() -> Self {
        Self {
            phantom: PhantomData::default(),
        }
    }
}

impl<R: tauri::Runtime, A: tauri::Assets> TauriPlugin<R, A> {
    fn runner(app: App) {
        let mut app = app;

        let settings = app
            .world
            .remove_non_send::<TauriPluginSettings<R, A>>()
            .expect("the Tauri plugin was not configured");

        let tauri_app = settings
            .builder
            .build(settings.context)
            .expect("failed to build Tauri application");

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

impl<R: tauri::Runtime, A: tauri::Assets> Plugin for TauriPlugin<R, A> {
    fn build(&self, app: &mut App) {
        app.set_runner(Self::runner);
    }
}
