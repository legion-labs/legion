use legion_app::{App, Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};
use legion_renderer::{Renderer, RendererSystemLabel};
use legion_window::Windows;

use crate::component::PresenterWindow;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            render_presenter_windows.system().after(RendererSystemLabel::Main)
        );        
    }
}

fn render_presenter_windows(
    windows : Res<Windows>,
    renderer : Res<Renderer>,
    mut pres_windows: Query<&mut PresenterWindow>
) {

    let graphics_queue = renderer.graphics_queue();

    for mut pres_window in pres_windows.iter_mut() {
        let wnd = windows.get( pres_window.window_id).unwrap();        
        pres_window.present(wnd, graphics_queue);
    }
}