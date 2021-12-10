use std::collections::HashMap;

use lgn_app::{App, AppExit, CoreStage, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_core::CorePlugin;
use lgn_ecs::prelude::*;
use lgn_input::keyboard::{KeyCode, KeyboardInput};
use lgn_input::mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel};
use lgn_input::InputPlugin;
use lgn_math::Mat3;
use lgn_presenter::offscreen_helper::Resolution;
use lgn_presenter_snapshot::component::PresenterSnapshot;
use lgn_presenter_window::component::PresenterWindow;
use lgn_renderer::components::{
    CameraComponent, RenderSurface, RenderSurfaceExtents, RenderSurfaceId, RotationComponent,
    StaticMesh,
};
use lgn_renderer::{Renderer, RendererPlugin, RendererSystemLabel};
use lgn_transform::components::Transform;
use lgn_window::{
    WindowCloseRequested, WindowCreated, WindowDescriptor, WindowId, WindowPlugin, WindowResized,
    Windows,
};
use lgn_winit::{WinitPlugin, WinitWindows};
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
    const ARG_NAME_WIDTH: &str = "width";
    const ARG_NAME_HEIGHT: &str = "height";
    const ARG_NAME_SNAPSHOT: &str = "snapshot";
    const ARG_NAME_SETUP_NAME: &str = "setup-name";
    const ARG_NAME_EGUI: &str = "egui";
    const ARG_NAME_USE_ASSET_REGISTRY: &str = "use-asset-registry";
    let matches = clap::App::new("graphics-sandbox")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Legion Labs")
        .about("A sandbox for graphics")
        .arg(
            clap::Arg::with_name(ARG_NAME_WIDTH)
                .short("w")
                .long(ARG_NAME_WIDTH)
                .help("The width of the window")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_NAME_HEIGHT)
                .short("h")
                .long(ARG_NAME_HEIGHT)
                .help("The height of the window")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_NAME_SNAPSHOT)
                .short("s")
                .long(ARG_NAME_SNAPSHOT)
                .help("Saves a snapshot of the scene")
                .takes_value(false),
        )
        .arg(
            clap::Arg::with_name(ARG_NAME_SETUP_NAME)
                .long(ARG_NAME_SETUP_NAME)
                .help("Name of the setup to launch")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name(ARG_NAME_USE_ASSET_REGISTRY)
                .long(ARG_NAME_USE_ASSET_REGISTRY)
                .takes_value(false)
                .help("Use asset registry data instead of a hardcoded scene"),
        )
        .arg(
            clap::Arg::with_name(ARG_NAME_EGUI)
                .long(ARG_NAME_EGUI)
                .takes_value(false)
                .help("Enable egui immediate mode GUI"),
        )
        .get_matches();

    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .init()
        .unwrap();

    let width = matches
        .value_of(ARG_NAME_WIDTH)
        .map(|s| s.parse::<f32>().unwrap())
        .unwrap_or(1280.0);
    let height = matches
        .value_of(ARG_NAME_HEIGHT)
        .map(|s| s.parse::<f32>().unwrap())
        .unwrap_or(720.0);
    let setup_name = matches
        .value_of(ARG_NAME_SETUP_NAME)
        .unwrap_or("simple-scene");

    let mut app = App::new();
    app.add_plugin(CorePlugin::default())
        .add_plugin(RendererPlugin::new(true, matches.is_present(ARG_NAME_EGUI)))
        .add_plugin(WindowPlugin::default())
        .add_plugin(InputPlugin::default());

    if matches.is_present(ARG_NAME_SNAPSHOT) {
        app.insert_resource(SnapshotDescriptor {
            setup_name: setup_name.to_string(),
            width,
            height,
        })
        .insert_resource(ScheduleRunnerSettings::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_system(presenter_snapshot_system.before(RendererSystemLabel::FrameUpdate))
        .add_system_to_stage(CoreStage::Last, on_snapshot_app_exit);
    } else {
        app.insert_resource(WindowDescriptor {
            width,
            height,
            ..WindowDescriptor::default()
        });
        app.add_plugin(WinitPlugin::default())
            .add_system(on_window_created.exclusive_system())
            .add_system(on_window_resized.exclusive_system())
            .add_system(on_window_close_requested.exclusive_system())
            .add_system(camera_control.system())
            .insert_resource(RenderSurfaces::new());
    }
    if matches.is_present(ARG_NAME_USE_ASSET_REGISTRY) {
        app.insert_resource(AssetRegistrySettings::default())
            .add_plugin(AssetRegistryPlugin::default());
    } else {
        app.add_startup_system(init_scene);
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
    } else if frame_counter.frame_count > frame_counter.frame_target {
        app_exit_events.send(AppExit);
    }
    frame_counter.frame_count += 1;
}

fn init_scene(mut commands: Commands) {
    // plane
    commands
        .spawn()
        .insert(Transform::from_xyz(-0.5, 0.0, 0.0))
        .insert(StaticMesh {
            mesh_id: 0,
            color: (0, 0, 255).into(),
            offset: 0,
        })
        .insert(RotationComponent {
            rotation_speed: (0.4, 0.0, 0.0),
        });

    // cube
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(StaticMesh {
            mesh_id: 1,
            color: (255, 0, 0).into(),
            offset: 0,
        })
        .insert(RotationComponent {
            rotation_speed: (0.0, 0.4, 0.0),
        });

    // pyramid
    commands
        .spawn()
        .insert(Transform::from_xyz(0.5, 0.0, 0.0))
        .insert(StaticMesh {
            mesh_id: 2,
            color: (0, 255, 0).into(),
            offset: 0,
        })
        .insert(RotationComponent {
            rotation_speed: (0.0, 0.0, 0.4),
        });

    // camera
    commands.spawn().insert(CameraComponent::default());
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

#[derive(Default)]
struct CameraMoving(bool);

fn camera_control(
    mut q_cameras: Query<'_, '_, &mut CameraComponent>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
    mut mouse_motion_events: EventReader<'_, '_, MouseMotion>,
    mut mouse_wheel_events: EventReader<'_, '_, MouseWheel>,
    mut mouse_button_input_events: EventReader<'_, '_, MouseButtonInput>,
    mut camera_moving: Local<CameraMoving>,
) {
    let mut q_cameras = q_cameras
        .iter_mut()
        .map(|v| v.into_inner())
        .collect::<Vec<&mut CameraComponent>>();

    for mouse_button_input_event in mouse_button_input_events.iter() {
        if mouse_button_input_event.button == MouseButton::Right {
            camera_moving.0 = mouse_button_input_event.state.is_pressed();
        }
    }

    if q_cameras.is_empty() || !camera_moving.0 {
        return;
    }

    for camera in q_cameras.iter_mut() {
        for keyboard_input_event in keyboard_input_events.iter() {
            if let Some(key_code) = keyboard_input_event.key_code {
                match key_code {
                    KeyCode::W => {
                        camera.pos += camera.dir * camera.speed / 60.0;
                    }
                    KeyCode::S => {
                        camera.pos -= camera.dir * camera.speed / 60.0;
                    }
                    KeyCode::D => {
                        let cross = camera.up.cross(camera.dir).normalize();
                        camera.pos += cross * camera.speed / 60.0;
                    }
                    KeyCode::A => {
                        let cross = camera.up.cross(camera.dir).normalize();
                        camera.pos -= cross * camera.speed / 60.0;
                    }
                    _ => {}
                }
            }
        }

        for mouse_motion_event in mouse_motion_events.iter() {
            let rotation_x = Mat3::from_axis_angle(
                camera.up,
                mouse_motion_event.delta.x * camera.rotation_speed / 60.0,
            );
            let rotation_y = Mat3::from_axis_angle(
                camera.up.cross(camera.dir).normalize(),
                mouse_motion_event.delta.y * camera.rotation_speed / 60.0,
            );
            camera.dir = rotation_x * rotation_y * camera.dir;
        }

        for mouse_wheel_event in mouse_wheel_events.iter() {
            camera.speed = (camera.speed * (1.0 + mouse_wheel_event.y * 0.1)).clamp(0.01, 10.0);
        }
    }
}
