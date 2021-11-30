use std::collections::HashMap;

use legion_app::{App, AppExit, CoreStage, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use legion_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use legion_async::AsyncPlugin;
use legion_core::CorePlugin;
use legion_ecs::prelude::*;
use legion_input::InputPlugin;
use legion_presenter::offscreen_helper::Resolution;
use legion_presenter_snapshot::component::PresenterSnapshot;
use legion_presenter_window::component::PresenterWindow;
use legion_renderer::components::{RenderSurface, RenderSurfaceExtents, RenderSurfaceId};
use legion_renderer::components::{RotationComponent, StaticMesh};
use legion_renderer::{Renderer, RendererPlugin, RendererSystemLabel};
use legion_transform::components::Transform;
use legion_window::{
    WindowCloseRequested, WindowCreated, WindowDescriptor, WindowId, WindowPlugin, WindowResized,
    Windows,
};
use legion_winit::{WinitPlugin, WinitWindows};
use log::LevelFilter;
use simple_logger::SimpleLogger;

struct RenderSurfaces {
    window_id_mapper: HashMap<WindowId, RenderSurfaceId>,
}

impl RenderSurfaces {
    pub fn new() -> Self {
        Self {
            window_id_mapper: HashMap::new(),
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

struct SnapshotDescriptor {
    setup_name: String,
    width: f32,
    height: f32,
}

#[derive(Default)]
struct SnapshotFrameCounter {
    frame_count: i32,
    frame_target: i32,
}

fn main() {
    let matches = clap::App::new("graphics-sandbox")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Legion Labs")
        .about("A sandbox for graphics")
        .arg(
            clap::Arg::with_name("width")
                .short("w")
                .long("width")
                .help("The width of the window")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("height")
                .short("h")
                .long("height")
                .help("The height of the window")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("snapshot")
                .short("s")
                .long("snapshot")
                .help("Saves a snapshot of the scene")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name("setup-name")
                .long("setup-name")
                .help("Name of the setup to launch")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("use-asset-registry")
                .takes_value(false)
                .help(""),
        )
        .get_matches();

    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .init()
        .unwrap();

    let width = matches
        .value_of("width")
        .map(|s| s.parse::<f32>().unwrap())
        .unwrap_or(1280.0);
    let height = matches
        .value_of("height")
        .map(|s| s.parse::<f32>().unwrap())
        .unwrap_or(720.0);
    let setup_name = matches.value_of("setup-name").unwrap_or("triangle");

    let mut app = App::new();
    app.add_plugin(CorePlugin::default())
        .add_plugin(AsyncPlugin {})
        .add_plugin(RendererPlugin::default());

    if matches.is_present("snapshot") {
        app.insert_resource(SnapshotDescriptor {
            setup_name: setup_name.to_string(),
            width,
            height,
        })
        .insert_resource(ScheduleRunnerSettings::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_system(
            presenter_snapshot_system
                .system()
                .before(RendererSystemLabel::FrameUpdate),
        )
        .add_system_to_stage(CoreStage::Last, on_snapshot_app_exit);
    } else {
        app.insert_resource(WindowDescriptor {
            width,
            height,
            ..WindowDescriptor::default()
        });
        app.add_plugin(WindowPlugin::default())
            .add_plugin(InputPlugin::default())
            .add_plugin(WinitPlugin::default())
            .add_system(on_window_created.exclusive_system())
            .add_system(on_window_resized.exclusive_system())
            .add_system(on_window_close_requested.exclusive_system())
            .insert_resource(RenderSurfaces::new());
    }
    if matches.is_present("use-asset-registry") {
        app.insert_resource(AssetRegistrySettings::default())
            .add_plugin(AssetRegistryPlugin::default());
    } else {
        app.add_startup_system(init_scene.system());
    }
    app.run();
}

fn on_window_created(
    mut commands: Commands,
    mut ev_wnd_created: EventReader<WindowCreated>,
    wnd_list: Res<Windows>,
    winit_wnd_list: Res<WinitWindows>,
    renderer: Res<Renderer>,
    mut render_surfaces: ResMut<RenderSurfaces>,
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
}

fn on_window_resized(
    mut ev_wnd_resized: EventReader<WindowResized>,
    wnd_list: Res<Windows>,
    renderer: Res<Renderer>,
    mut q_render_surfaces: Query<&mut RenderSurface>,
    render_surfaces: Res<RenderSurfaces>,
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
}

fn on_window_close_requested(
    mut commands: Commands,
    mut ev_wnd_destroyed: EventReader<WindowCloseRequested>,
    query_render_surface: Query<(Entity, &RenderSurface)>,
    mut render_surfaces: ResMut<RenderSurfaces>,
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
}

fn presenter_snapshot_system(
    mut commands: Commands,
    snapshot_descriptor: Res<SnapshotDescriptor>,
    renderer: Res<Renderer>,
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
    mut frame_counter: Local<SnapshotFrameCounter>,
) {
    if frame_counter.frame_count == 0 {
        let mut render_surface = RenderSurface::new(
            &renderer,
            RenderSurfaceExtents::new(
                snapshot_descriptor.width as u32,
                snapshot_descriptor.height as u32,
            ),
        );
        let render_surface_id = render_surface.id();

        render_surface.register_presenter(|| {
            PresenterSnapshot::new(
                &snapshot_descriptor.setup_name,
                frame_counter.frame_target,
                renderer.into_inner(),
                render_surface_id,
                Resolution::new(
                    snapshot_descriptor.width as u32,
                    snapshot_descriptor.height as u32,
                ),
            )
            .unwrap()
        });

        commands.spawn().insert(render_surface);
    } else {
        if frame_counter.frame_count > frame_counter.frame_target {
            app_exit_events.send(AppExit);
        }
    }
    frame_counter.frame_count += 1;
}

fn init_scene(mut commands: Commands) {
    // plane
    commands
        .spawn()
        .insert(Transform::from_xyz(-0.5, 0.0, 0.0))
        .insert(StaticMesh { mesh_id: 0 })
        .insert(RotationComponent {
            rotation_speed: (0.4, 0.0, 0.0),
        });

    // cube
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(StaticMesh { mesh_id: 1 })
        .insert(RotationComponent {
            rotation_speed: (0.0, 0.4, 0.0),
        });

    // pyramid
    commands
        .spawn()
        .insert(Transform::from_xyz(0.5, 0.0, 0.0))
        .insert(StaticMesh { mesh_id: 2 })
        .insert(RotationComponent {
            rotation_speed: (0.0, 0.0, 0.4),
        });
}

fn on_snapshot_app_exit(
    mut commands: Commands,
    mut app_exit: EventReader<AppExit>,
    query_render_surface: Query<(Entity, &RenderSurface)>,
) {
    if app_exit.iter().last().is_some() {
        for (entity, _) in query_render_surface.iter() {
            commands.entity(entity).despawn();
        }
    }
}
