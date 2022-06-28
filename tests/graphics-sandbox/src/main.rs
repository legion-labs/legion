//! Test sandbox for graphics programmers

#![allow(clippy::needless_pass_by_value)]

use std::{path::PathBuf, sync::Arc};

use clap::Parser;

use lgn_app::{prelude::*, AppExit, ScheduleRunnerPlugin};
use lgn_asset_registry::{AssetRegistryPlugin, AssetRegistrySettings};
use lgn_core::CorePlugin;
use lgn_data_runtime::{AssetRegistry, ResourceTypeAndId};
use lgn_ecs::{event::Events, prelude::*};
use lgn_gilrs::GilrsPlugin;
use lgn_graphics_data::{Color, GraphicsPlugin};
use lgn_graphics_renderer::{
    components::{
        LightComponent, LightType, RenderSurface, RenderSurfaceCreatedForWindow,
        RenderSurfaceExtents, RenderSurfaces, VisualComponent,
    },
    labels::RendererLabel,
    resources::{
        PipelineManager, RenderModel, CONE_MODEL_RESOURCE_ID, CUBE_MODEL_RESOURCE_ID,
        CYLINDER_MODEL_RESOURCE_ID, PLANE_MODEL_RESOURCE_ID, PYRAMID_MODEL_RESOURCE_ID,
        SPHERE_MODEL_RESOURCE_ID, TORUS_MODEL_RESOURCE_ID,
    },
    {Renderer, RendererPlugin},
};
use lgn_hierarchy::HierarchyPlugin;
use lgn_input::{
    keyboard::{KeyCode, KeyboardInput},
    InputPlugin,
};
use lgn_presenter_snapshot::{component::PresenterSnapshot, PresenterSnapshotPlugin};
use lgn_presenter_window::component::PresenterWindow;
use lgn_scene_plugin::ScenePlugin;
use lgn_time::TimePlugin;
use lgn_tracing::{flush_monitor::FlushMonitor, warn, LevelFilter};
use lgn_transform::prelude::{Transform, TransformBundle, TransformPlugin};
use lgn_window::{WindowCloseRequested, WindowDescriptor, WindowPlugin, Windows};
use lgn_winit::{WinitPlugin, WinitSettings, WinitWindows};
use sample_data::SampleDataPlugin;

mod meta_cube_test;
pub(crate) use meta_cube_test::*;

struct SnapshotDescriptor {
    setup_name: String,
    width: f32,
    height: f32,
}

struct SnapshotFrameCounter {
    frame_count: i32,
    frame_target: i32,
}

impl Default for SnapshotFrameCounter {
    fn default() -> Self {
        Self {
            frame_count: 0,
            frame_target: 1,
        }
    }
}

#[derive(Parser, Default)]
#[clap(name = "graphics-sandbox")]
#[clap(about = "A sandbox for graphics", version, author)]
struct Args {
    /// Verbose
    #[clap(short, long)]
    verbose: bool,
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
    #[clap(long)]
    setup_name: Option<String>,
    /// Root object to load, usually a world
    #[clap(long)]
    root: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut telemetry_guard_builder = lgn_telemetry_sink::TelemetryGuardBuilder::default();
    if args.verbose {
        telemetry_guard_builder =
            telemetry_guard_builder.with_max_level_override(LevelFilter::Trace);
    }

    let mut app = App::new(telemetry_guard_builder);

    let root_asset = if args.setup_name.is_none() {
        args.root
            .as_deref()
            .unwrap_or("(1d9ddd99aad89045,af7e6ef0-c271-565b-c27a-b8cd93c3546a)")
            .parse::<ResourceTypeAndId>()
            .ok()
    } else {
        None
    };

    let project_folder =
        lgn_config::get_or("editor_srv.project_dir", PathBuf::from("tests/sample-data")).unwrap();

    let asset_registry_settings = AssetRegistrySettings::new(
        Some(project_folder.join("runtime").join("game.manifest")),
        root_asset.into_iter().collect::<Vec<_>>(),
    );
    app.insert_resource(asset_registry_settings)
        .add_plugin(lgn_async::AsyncPlugin::default())
        .add_plugin(AssetRegistryPlugin::default())
        .add_plugin(GraphicsPlugin::default())
        .add_plugin(SampleDataPlugin::default())
        .add_plugin(ScenePlugin::new(root_asset));

    app.add_plugin(CorePlugin::default())
        .add_plugin(TimePlugin::default())
        .add_plugin(RendererPlugin::default())
        .insert_resource(WindowDescriptor {
            width: args.width,
            height: args.height,
            ..WindowDescriptor::default()
        })
        .insert_resource(FlushMonitor::new(5))
        .add_plugin(WindowPlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(HierarchyPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(GilrsPlugin::default())
        .add_system_to_stage(CoreStage::Last, tick_flush_monitor);

    if args.snapshot {
        app.insert_resource(SnapshotDescriptor {
            setup_name: args
                .setup_name
                .clone()
                .unwrap_or_else(|| "default".to_owned()),
            width: args.width,
            height: args.height,
        })
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(PresenterSnapshotPlugin::default())
        .add_system(presenter_snapshot_system)
        .add_system_to_stage(CoreStage::Last, on_snapshot_app_exit);
    } else {
        app.insert_resource(WinitSettings {
            return_from_run: true,
            ..WinitSettings::default()
        })
        .add_plugin(WinitPlugin::default())
        .add_system(on_render_surface_created_for_window.exclusive_system());
    }

    app.add_system_to_stage(CoreStage::Last, check_keyboard_events);

    match args.setup_name.as_deref() {
        Some("light-test") => {
            app.add_startup_system_to_stage(
                StartupStage::PostStartup,
                init_light_test.after(RendererLabel::DefaultResourcesInstalled),
            );
        }

        Some("stress-test") => {
            app.add_plugin(MetaCubePlugin::new());
        }

        Some("simple-scene") => {
            app.add_startup_system_to_stage(
                StartupStage::PostStartup,
                init_scene.after(RendererLabel::DefaultResourcesInstalled),
            );
        }

        Some(_) => {
            warn!("Unknow setup: {}", args.setup_name.unwrap());
        }

        None => (),
    }

    app.run();
}

fn on_render_surface_created_for_window(
    mut event_render_surface_created: EventReader<'_, '_, RenderSurfaceCreatedForWindow>,
    wnd_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    winit_wnd_list: NonSend<'_, WinitWindows>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for event in event_render_surface_created.iter() {
        let render_surface = render_surfaces.get_from_window_id_mut(event.window_id);
        let wnd = wnd_list.get(event.window_id).unwrap();
        let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());

        let winit_wnd = winit_wnd_list.get_window(event.window_id).unwrap();
        render_surface
            .register_presenter(|| PresenterWindow::from_window(&renderer, winit_wnd, extents));
    }
}

fn presenter_snapshot_system(
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
    snapshot_descriptor: Res<'_, SnapshotDescriptor>,
    renderer: Res<'_, Renderer>,
    pipeline_manager: Res<'_, PipelineManager>,
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
    mut frame_counter: Local<'_, SnapshotFrameCounter>,
) {
    if frame_counter.frame_count == 0 {
        let mut render_surface = RenderSurface::new_offscreen_window(
            &renderer,
            RenderSurfaceExtents::new(
                snapshot_descriptor.width as u32,
                snapshot_descriptor.height as u32,
            ),
        );

        render_surface.add_default_viewport();

        let device_context = renderer.device_context();

        render_surface.register_presenter(|| {
            PresenterSnapshot::new(
                &snapshot_descriptor.setup_name,
                frame_counter.frame_target,
                device_context,
                &pipeline_manager,
                RenderSurfaceExtents::new(
                    snapshot_descriptor.width as u32,
                    snapshot_descriptor.height as u32,
                ),
            )
        });

        render_surfaces.insert(render_surface);
    } else if frame_counter.frame_count > frame_counter.frame_target {
        app_exit_events.send(AppExit);
    }
    frame_counter.frame_count += 1;
}

fn init_light_test(mut commands: Commands<'_, '_>, asset_registry: Res<'_, Arc<AssetRegistry>>) {
    // sphere 1
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -0.5, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&SPHERE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (255, 0, 0).into(),
            1.0,
        ));

    // sphere 2
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.5, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&SPHERE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (0, 255, 0).into(),
            1.0,
        ));

    // sphere 3
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&SPHERE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (0, 0, 255).into(),
            1.0,
        ));

    // directional light
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 1.0,
        )))
        .insert(LightComponent {
            light_type: LightType::Directional,
            radiance: 40.0,
            color: Color::WHITE,
            enabled: false,
            ..LightComponent::default()
        });

    // omnidirectional light 1
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            1.0, 0.0, 1.0,
        )))
        .insert(LightComponent {
            light_type: LightType::OmniDirectional,
            radiance: 40.0,
            color: Color::WHITE,
            enabled: false,
            ..LightComponent::default()
        });

    // omnidirectional light 2
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -1.0, 0.0, 1.0,
        )))
        .insert(LightComponent {
            light_type: LightType::OmniDirectional,
            radiance: 40.0,
            color: Color::WHITE,
            enabled: false,
            ..LightComponent::default()
        });

    // spotlight
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 1.0,
        )))
        .insert(LightComponent {
            light_type: LightType::Spot,
            radiance: 40.0,
            cone_angle: std::f32::consts::PI / 4.0,
            color: Color::WHITE,
            enabled: true,
            ..LightComponent::default()
        });
}

fn init_scene(mut commands: Commands<'_, '_>, asset_registry: Res<'_, Arc<AssetRegistry>>) {
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.0, 0.0, 0.1,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&PLANE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (255, 0, 0).into(),
            1.0,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -0.5, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&CUBE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (0, 255, 0).into(),
            1.0,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            0.5, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&PYRAMID_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (0, 0, 255).into(),
            1.0,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            1.0, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&SPHERE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (255, 0, 255).into(),
            1.0,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -1.0, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&CYLINDER_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (0, 255, 255).into(),
            1.0,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            1.5, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&TORUS_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (255, 255, 0).into(),
            1.0,
        ));

    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            -1.5, 0.0, 0.0,
        )))
        .insert(VisualComponent::new(
            &asset_registry
                .lookup::<RenderModel>(&CONE_MODEL_RESOURCE_ID)
                .expect("Must be loaded"),
            (128, 128, 255).into(),
            1.0,
        ));

    // omnidirectional light
    commands
        .spawn()
        .insert_bundle(TransformBundle::from_transform(Transform::from_xyz(
            1.0, 0.0, 1.0,
        )))
        .insert(LightComponent {
            light_type: LightType::Directional,
            radiance: 10.0,
            color: Color::new(127, 127, 127, 255),
            enabled: true,
            ..LightComponent::default()
        });
}

fn on_snapshot_app_exit(
    mut app_exit: EventReader<'_, '_, AppExit>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    if app_exit.iter().last().is_some() {
        render_surfaces.clear();
    }
}

fn tick_flush_monitor(flush_monitor: Res<'_, FlushMonitor>) {
    flush_monitor.tick();
}

fn check_keyboard_events(
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
    mut window_close_requested_events: ResMut<'_, Events<WindowCloseRequested>>,
    windows: Res<'_, Windows>,
) {
    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key_code) = keyboard_input_event.key_code {
            if key_code == KeyCode::Escape && keyboard_input_event.state.is_pressed() {
                window_close_requested_events.send(WindowCloseRequested {
                    id: windows.get_primary().unwrap().id(),
                });
            }
        }
    }
}
