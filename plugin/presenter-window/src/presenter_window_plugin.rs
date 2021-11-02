#![allow(clippy::needless_pass_by_value)]

use legion_app::{App, Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};
use legion_renderer::{components::RenderSurface, Renderer, RendererSystemLabel};
use legion_window::Windows;

use crate::component::PresenterWindow;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            render_presenter_windows
                .system()
                .after(RendererSystemLabel::Main),
        );
    }
}

fn render_presenter_windows(
    windows: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut pres_windows: Query<'_, '_, &mut PresenterWindow>,
    mut render_surfaces: Query<'_, '_, &mut RenderSurface>,
) {
    let graphics_queue = renderer.graphics_queue();
    let wait_sem = renderer.frame_signal_semaphore();

    for mut pres_window in pres_windows.iter_mut() {
        let wnd = windows.get(pres_window.window_id()).unwrap();
        if wnd.physical_width() > 0 && wnd.physical_height() > 0 {
            let render_surface = render_surfaces
                .iter_mut()
                .find(|x| pres_window.render_surface_id().eq(&x.id()))
                .map(Mut::into_inner);

            pres_window.present(wnd, graphics_queue, wait_sem, render_surface);
        }
    }
}
