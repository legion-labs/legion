use lgn_app::prelude::*;
use lgn_ecs::prelude::*;
use lgn_presenter_window::component::PresenterWindow;
use lgn_renderer::{
    components::{RenderSurface, RenderSurfaceExtents, RenderSurfaceId},
    Renderer,
};
use lgn_utils::HashMap;
use lgn_window::{
    WindowCloseRequested, WindowCreated, WindowDescriptor, WindowId, WindowPlugin, WindowResized,
    Windows,
};
use lgn_winit::{WinitPlugin, WinitWindows};

pub(crate) fn build_standalone(app: &mut App) -> &mut App {
    let width = 1280_f32;
    let height = 720_f32;
    app.insert_resource(WindowDescriptor {
        width,
        height,
        ..WindowDescriptor::default()
    })
    .add_plugin(WindowPlugin::default())
    .add_plugin(WinitPlugin::default())
    .add_system(on_window_created.exclusive_system())
    .add_system(on_window_resized.exclusive_system())
    .add_system(on_window_close_requested.exclusive_system())
    .insert_resource(RenderSurfaces::new())
}

fn on_window_created(
    mut commands: Commands<'_, '_>,
    mut ev_wnd_created: EventReader<'_, '_, WindowCreated>,
    wnd_list: Res<'_, Windows>,
    winit_wnd_list: Res<'_, WinitWindows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_created.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();
        let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());
        let mut render_surface = RenderSurface::new(&renderer, extents);
        render_surfaces.insert(ev.id, render_surface.id());
        let winit_wnd = winit_wnd_list.get_window(ev.id).unwrap();
        render_surface
            .register_presenter(|| PresenterWindow::from_window(&renderer, winit_wnd, extents));
        commands.spawn().insert(render_surface);
    }

    drop(wnd_list);
    drop(winit_wnd_list);
    drop(renderer);
}

fn on_window_resized(
    mut ev_wnd_resized: EventReader<'_, '_, WindowResized>,
    wnd_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    render_surfaces: Res<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_resized.iter() {
        let render_surface_id = render_surfaces.get_from_window_id(ev.id);
        if let Some(render_surface_id) = render_surface_id {
            let render_surface = q_render_surfaces
                .iter_mut()
                .find(|x| x.id() == *render_surface_id);
            if let Some(mut render_surface) = render_surface {
                let wnd = wnd_list.get(ev.id).unwrap();
                render_surface.resize(
                    &renderer,
                    RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height()),
                );
            }
        }
    }

    drop(wnd_list);
    drop(renderer);
    drop(render_surfaces);
}

fn on_window_close_requested(
    mut commands: Commands<'_, '_>,
    mut ev_wnd_destroyed: EventReader<'_, '_, WindowCloseRequested>,
    query_render_surface: Query<'_, '_, (Entity, &RenderSurface)>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_destroyed.iter() {
        let render_surface_id = render_surfaces.get_from_window_id(ev.id);
        if let Some(render_surface_id) = render_surface_id {
            let query_result = query_render_surface
                .iter()
                .find(|x| x.1.id() == *render_surface_id);
            if let Some(query_result) = query_result {
                commands.entity(query_result.0).despawn();
            }
        }
        render_surfaces.remove(ev.id);
    }

    drop(query_render_surface);
}

struct RenderSurfaces {
    window_id_mapper: HashMap<WindowId, RenderSurfaceId>,
}

impl RenderSurfaces {
    pub fn new() -> Self {
        Self {
            window_id_mapper: HashMap::default(),
        }
    }

    pub fn insert(&mut self, window_id: WindowId, render_surface_id: RenderSurfaceId) {
        let result = self.window_id_mapper.insert(window_id, render_surface_id);
        assert!(result.is_none());
    }

    pub fn remove(&mut self, window_id: WindowId) {
        let result = self.window_id_mapper.remove(&window_id);
        assert!(result.is_some());
    }

    pub fn get_from_window_id(&self, window_id: WindowId) -> Option<&RenderSurfaceId> {
        self.window_id_mapper.get(&window_id)
    }
}
