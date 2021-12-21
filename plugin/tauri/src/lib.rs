//! Tauri plugin for Legion's ECS.
//!
//! Provides Tauri integration into Legion's ECS.

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
#![allow()]

use std::sync::Mutex;

use lgn_app::prelude::*;
pub use lgn_tauri_macros::*;

pub struct TauriPluginSettings<R: tauri::Runtime> {
    builder: tauri::Builder<R>,
}

impl<R: tauri::Runtime> TauriPluginSettings<R> {
    pub fn new(builder: tauri::Builder<R>) -> Self {
        Self { builder }
    }
}

/// Provides game-engine integration into Tauri's event loop.
pub struct TauriPlugin<A: tauri::Assets> {
    context: Mutex<Option<tauri::Context<A>>>,
}

impl<A: tauri::Assets> TauriPlugin<A> {
    pub fn new(context: tauri::Context<A>) -> Self {
        Self {
            context: Mutex::new(Some(context)),
        }
    }
}

impl<A: tauri::Assets> Plugin for TauriPlugin<A> {
    fn build(&self, app: &mut AppBuilder) {
        let context = std::mem::replace(&mut *self.context.lock().unwrap(), None).unwrap();

        app.set_runner(move |app| {
            let mut app = app;

            let settings = app
                .world
                .remove_non_send::<TauriPluginSettings<tauri::Wry>>()
                .expect("the Tauri plugin was not configured");

            let tauri_app = settings
                .builder
                .build(context)
                .expect("failed to build Tauri application");

            // FIXME: Once https://github.com/tauri-apps/tauri/pull/2667 is merged, we can
            // get rid of this and move the value directly instead.
            let app = std::rc::Rc::new(std::cell::RefCell::new(app));

            tauri_app.run(move |_, event| {
                if let tauri::Event::MainEventsCleared = event {
                    app.borrow_mut().update();
                }
            });
        });
    }
}
