//! Presenter plugin made for windowing system.

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

use legion_app::{App, Plugin};

pub mod component;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_system_to_stage(
        //     CoreStage::PostUpdate,
        //     render_presenter_windows
        //         .system()
        //         .after(RendererSystemLabel::FrameDone),
        // );
    }
}
/*
#[allow(clippy::needless_pass_by_value)]
fn render_presenter_windows(
    windows: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut pres_windows: Query<'_, '_, &mut PresenterWindow>,
    mut render_surfaces: Query<'_, '_, &mut RenderSurface>,
) {
    let mut render_context = RenderContext::new(&renderer);
    // let mut graphics_queue = renderer.queue_mut(QueueType::Graphics);
    // let wait_sem = renderer.frame_signal_semaphore();

    for mut pres_window in pres_windows.iter_mut() {
        let wnd = windows.get(pres_window.window_id()).unwrap();
        if wnd.physical_width() > 0 && wnd.physical_height() > 0 {
            let render_surface = render_surfaces
                .iter_mut()
                .find(|x| pres_window.render_surface_id().eq(&x.id()))
                .map(Mut::into_inner);

            pres_window.present_(
                &mut render_context,
                wnd,
                // &mut graphics_queue,
                render_surface,
            );
        }
    }
}
*/
