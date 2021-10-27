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
    windows: Res<Windows>,
    renderer: Res<Renderer>,
    mut pres_windows: Query<&mut PresenterWindow>,
    render_surfaces: Query<&RenderSurface>,
) {
    let graphics_queue = renderer.graphics_queue();
    let wait_sem = renderer.frame_signal_semaphore();

    for mut pres_window in pres_windows.iter_mut() {
        let wnd = windows.get(pres_window.window_id).unwrap();

        let src_texture_view = render_surfaces
            .iter()
            .find(|x| pres_window.render_surface_id.eq(&x.id))
            .map(|x| &x.texture_srv);

        pres_window.present(wnd, graphics_queue, wait_sem, src_texture_view);
    }
}
