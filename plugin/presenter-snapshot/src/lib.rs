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
pub use legion_presenter::offscreen_helper::Resolution;

#[derive(Default)]
pub struct PresenterSnapshotPlugin;

impl Plugin for PresenterSnapshotPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_system_to_stage(
        //     CoreStage::PostUpdate,
        //     render_presenter_snapshots
        //         .system()
        //         .after(RendererSystemLabel::FrameDone),
        // );
    }
}
/*
#[allow(clippy::needless_pass_by_value)]
fn render_presenter_snapshots(
    renderer: Res<'_, Renderer>,
    mut q_pres_snapshots: Query<'_, '_, &mut PresenterSnapshot>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
) {
    let mut render_context = RenderContext::new(&renderer);
    // let graphics_queue = renderer.queue(QueueType::Graphics);
    // let transient_descriptor_heap = render_context.transient_descriptor_heap();
    // let wait_sem = renderer.frame_signal_semaphore();

    for mut pres_snapshot in q_pres_snapshots.iter_mut() {
        // this loop is wrong, it's wip code, we need to add some snapshot render_surface mapping
        let render_surface = q_render_surfaces
            .iter_mut()
            .find(|x| pres_snapshot.render_surface_id().eq(&x.id()))
            .map(Mut::into_inner);

        if let Some(render_surface) = render_surface {
            if pres_snapshot
                .present(
                    &mut render_context,
                    // graphics_queue,
                    // transient_descriptor_heap,
                    // wait_sem,
                    render_surface,
                )
                .unwrap()
            {
                app_exit_events.send(AppExit);
            }
        }
    }
}
*/
