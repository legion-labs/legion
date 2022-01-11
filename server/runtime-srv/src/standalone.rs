use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_presenter_window::component::PresenterWindow;
use lgn_renderer::{
    components::{
        RenderSurface, RenderSurfaceCreatedForWindow, RenderSurfaceExtents, RenderSurfaces,
    },
    Renderer,
};
use lgn_window::{WindowDescriptor, WindowPlugin, Windows};
use lgn_winit::{WinitConfig, WinitPlugin, WinitWindows};

pub(crate) fn build_standalone(app: &mut App) -> &mut App {
    let width = 1280_f32;
    let height = 720_f32;
    app.insert_resource(WindowDescriptor {
        width,
        height,
        ..WindowDescriptor::default()
    })
    .insert_resource(WinitConfig {
        return_from_run: true,
    })
    .add_plugin(WindowPlugin::default())
    .add_plugin(WinitPlugin::default())
    .add_system(on_render_surface_created_for_window.exclusive_system())
    .insert_resource(RenderSurfaces::new())
}

#[allow(clippy::needless_pass_by_value)]
fn on_render_surface_created_for_window(
    mut event_render_surface_created: EventReader<'_, '_, RenderSurfaceCreatedForWindow>,
    wnd_list: Res<'_, Windows>,
    winit_wnd_list: Res<'_, WinitWindows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: Query<'_, '_, &mut RenderSurface>,
) {
    for event in event_render_surface_created.iter() {
        let render_surface = render_surfaces
            .iter_mut()
            .find(|x| x.id() == event.render_surface_id);
        if let Some(mut render_surface) = render_surface {
            let wnd = wnd_list.get(event.window_id).unwrap();
            let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());

            let winit_wnd = winit_wnd_list.get_window(event.window_id).unwrap();
            render_surface
                .register_presenter(|| PresenterWindow::from_window(&renderer, winit_wnd, extents));
        }
    }
}
