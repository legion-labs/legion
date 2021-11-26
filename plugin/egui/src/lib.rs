//! egui wrapper plugin

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

use egui::{CtxRef, Event, RawInput};
use legion_app::prelude::*;
use legion_ecs::prelude::*;
use legion_input::mouse::MouseMotion;

pub struct Egui {
    pub ctx: egui::CtxRef,
}

#[derive(Default)]
pub struct EguiPlugin {}

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        let mut ctx = egui::CtxRef::default();
        // Empty run to initialize the font texture
        ctx.begin_frame(RawInput::default());
        ctx.end_frame();

        app.insert_resource(Egui { ctx })
            .add_system_to_stage(CoreStage::PreUpdate, begin_frame.system());
    }
}

fn begin_frame(mut egui: ResMut<'_, Egui>, mut mouse_movements: EventReader<'_, '_, MouseMotion>) {
    // TODO: proper input
    let mut events: Vec<Event> = Vec::new();
    for mouse_movement in mouse_movements.iter() {
        events.push(Event::PointerMoved(egui::pos2(
            mouse_movement.delta.x,
            mouse_movement.delta.y,
        )));
    }
    let input = RawInput {
        events,
        ..RawInput::default()
    };
    egui.ctx.begin_frame(input);
}
