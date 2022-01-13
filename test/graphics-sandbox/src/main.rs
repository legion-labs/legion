//! Test sandbox for graphics programmers

#![allow(clippy::needless_pass_by_value)]

use clap::{AppSettings, Parser};

use lgn_app::{prelude::*, AppExit, ScheduleRunnerPlugin, ScheduleRunnerSettings};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_core::CorePlugin;
use lgn_ecs::prelude::*;
use lgn_input::InputPlugin;
use lgn_presenter::offscreen_helper::Resolution;
use lgn_presenter_snapshot::component::PresenterSnapshot;
use lgn_presenter_window::component::PresenterWindow;
use lgn_renderer::{
    components::{
        LightComponent, LightType, RenderSurface, RenderSurfaceCreatedForWindow,
        RenderSurfaceExtents, RotationComponent, StaticMesh,
    },
    resources::{DefaultMeshId, DefaultMeshes},
    {Renderer, RendererPlugin},
};
use lgn_transform::components::Transform;
use lgn_window::{WindowDescriptor, WindowPlugin, Windows};
use lgn_winit::{WinitConfig, WinitPlugin, WinitWindows};

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

#[derive(Parser, Default)]
#[clap(name = "graphics-sandbox")]
#[clap(about = "A sandbox for graphics", version, author)]
#[clap(setting(AppSettings::ArgRequiredElseHelp))]
struct Args {
    /// The width of the window
    #[clap(short, long, default_value_t = 1280.0)]
    width: f32,
    /// The height of the window
    #[clap(short, long, default_value_t = 720.0)]
    height: f32,
    /// Saves a snapshot of the scene
    #[clap(short, long)]
    snapshot: bool,
    /// Name of the setup to launch
    #[clap(long, default_value = "simple-scene")]
    setup_name: String,
    /// Use asset registry data instead of a hardcoded scene
    #[clap(long)]
    use_asset_registry: bool,
    /// Enable egui immediate mode GUI
    #[clap(long)]
    egui: bool,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();
    app.add_plugin(CorePlugin::default())
        .add_plugin(RendererPlugin::new(args.egui, !args.snapshot))
        .insert_resource(WindowDescriptor {
            width: args.width,
            height: args.height,
            ..WindowDescriptor::default()
        })
        .add_plugin(WindowPlugin::default())
        .add_plugin(InputPlugin::default());

    if args.snapshot {
        app.insert_resource(SnapshotDescriptor {
            setup_name: args.setup_name.clone(),
            width: args.width,
            height: args.height,
        })
        .insert_resource(ScheduleRunnerSettings::default())
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_system(presenter_snapshot_system)
        .add_system_to_stage(CoreStage::Last, on_snapshot_app_exit);
    } else {
        app.insert_resource(WinitConfig {
            return_from_run: true,
        })
        .add_plugin(WinitPlugin::default())
        .add_system(on_render_surface_created_for_window.exclusive_system());
    }
    if args.use_asset_registry {
        app.insert_resource(AssetRegistrySettings::default())
            .add_plugin(AssetRegistryPlugin::default())
            .add_plugin(generic_data::plugin::GenericDataPlugin::default())
            .add_startup_system(register_asset_loaders);
    } else if args.setup_name.eq("light_test") {
        app.add_startup_system(init_light_test);
    } else {
        app.add_startup_system(init_scene);
    }
    app.run();
}

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

fn presenter_snapshot_system(
    mut commands: Commands<'_, '_>,
    snapshot_descriptor: Res<'_, SnapshotDescriptor>,
    renderer: Res<'_, Renderer>,
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
    mut frame_counter: Local<'_, SnapshotFrameCounter>,
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

fn init_light_test(mut commands: Commands<'_, '_>, default_meshes: Res<'_, DefaultMeshes>) {
    // sphere 1
    commands
        .spawn()
        .insert(Transform::from_xyz(-0.5, 0.0, 0.0))
        .insert(StaticMesh::from_default_meshes(
            default_meshes.as_ref(),
            DefaultMeshId::Sphere as usize,
            (255, 0, 0).into(),
        ));

    // sphere 2
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(StaticMesh::from_default_meshes(
            default_meshes.as_ref(),
            DefaultMeshId::Sphere as usize,
            (0, 255, 0).into(),
        ));

    // sphere 3
    commands
        .spawn()
        .insert(Transform::from_xyz(0.5, 0.0, 0.0))
        .insert(StaticMesh::from_default_meshes(
            default_meshes.as_ref(),
            DefaultMeshId::Sphere as usize,
            (0, 0, 255).into(),
        ));

    // directional light
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 1.0, 0.0))
        .insert(LightComponent {
            light_type: LightType::Directional,
            radiance: 40.0,
            color: (1.0, 1.0, 1.0),
            enabled: false,
            ..LightComponent::default()
        });

    // omnidirectional light 1
    commands
        .spawn()
        .insert(Transform::from_xyz(1.0, 1.0, 0.0))
        .insert(LightComponent {
            light_type: LightType::Omnidirectional,
            radiance: 40.0,
            color: (1.0, 1.0, 1.0),
            enabled: false,
            ..LightComponent::default()
        });

    // omnidirectional light 2
    commands
        .spawn()
        .insert(Transform::from_xyz(-1.0, 1.0, 0.0))
        .insert(LightComponent {
            light_type: LightType::Omnidirectional,
            radiance: 40.0,
            color: (1.0, 1.0, 1.0),
            enabled: false,
            ..LightComponent::default()
        });

    // spotlight
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 1.0, 0.0))
        .insert(LightComponent {
            light_type: LightType::Spotlight {
                cone_angle: std::f32::consts::PI / 4.0,
            },
            radiance: 40.0,
            color: (1.0, 1.0, 1.0),
            enabled: true,
            ..LightComponent::default()
        });
}

fn init_scene(mut commands: Commands<'_, '_>, default_meshes: Res<'_, DefaultMeshes>) {
    // plane
    commands
        .spawn()
        .insert(Transform::from_xyz(-0.5, -0.1, 0.0))
        .insert(StaticMesh::from_default_meshes(
            default_meshes.as_ref(),
            DefaultMeshId::Plane as usize,
            (255, 0, 0).into(),
        ))
        .insert(RotationComponent {
            rotation_speed: (0.4, 0.0, 0.0),
        });

    // cube
    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, 0.0))
        .insert(StaticMesh::from_default_meshes(
            default_meshes.as_ref(),
            DefaultMeshId::Cube as usize,
            (0, 255, 0).into(),
        ));

    // pyramid
    commands
        .spawn()
        .insert(Transform::from_xyz(0.5, 0.0, 0.0))
        .insert(StaticMesh::from_default_meshes(
            default_meshes.as_ref(),
            DefaultMeshId::Pyramid as usize,
            (0, 0, 255).into(),
        ));

    // omnidirectional light
    commands
        .spawn()
        .insert(Transform::from_xyz(1.0, 1.0, 0.0))
        .insert(LightComponent {
            light_type: LightType::Omnidirectional,
            radiance: 40.0,
            color: (1.0, 1.0, 1.0),
            enabled: true,
            ..LightComponent::default()
        });
}

fn on_snapshot_app_exit(
    mut commands: Commands<'_, '_>,
    mut app_exit: EventReader<'_, '_, AppExit>,
    query_render_surface: Query<'_, '_, (Entity, &RenderSurface)>,
) {
    if app_exit.iter().last().is_some() {
        for (entity, _) in query_render_surface.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn register_asset_loaders(mut registry: NonSendMut<'_, lgn_data_runtime::AssetRegistryOptions>) {
    sample_data_runtime::add_loaders(&mut registry);
    lgn_graphics_runtime::add_loaders(&mut registry);
}
