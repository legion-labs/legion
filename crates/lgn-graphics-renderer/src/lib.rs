//! Renderer plugin.

// crate-specific lint exceptions:
#![allow(
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::new_without_default,
    clippy::uninit_vec
)]

mod cgen {
    include!(concat!(env!("OUT_DIR"), "/rust/mod.rs"));
}

use crate::components::{
    reflect_camera_components, reflect_viewports, reflect_visual_components, tmp_create_camera,
    tmp_debug_display_lights, EcsToRenderCamera, EcsToRenderLight, EcsToRenderViewport,
    EcsToRenderVisual,
};
use crate::core::{
    RenderCamera, RenderCommandQueuePool, RenderFeatures, RenderFeaturesBuilder,
    RenderGraphPersistentState, RenderLayerBuilder, RenderLayers, RenderObjects, RenderViewport,
    RenderViewportPrivateDataHandler, RenderViewportRendererData, Viewport, RENDER_LAYER_DEPTH,
    RENDER_LAYER_OPAQUE, RENDER_LAYER_PICKING,
};
use crate::features::{ModelFeature, RenderVisual};
use crate::lighting::{RenderLight, RenderLightTestData};
use crate::surface_renderer::SurfaceRenderer;

use std::sync::Arc;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};

use bumpalo_herd::Herd;
#[allow(unused_imports, clippy::wildcard_imports)]
use cgen::*;

pub mod labels;

use gpu_renderer::GpuInstanceManager;

pub use labels::*;

mod asset_to_ecs;
mod renderer;
use lgn_embedded_fs::EMBEDDED_FS;
use lgn_graphics_api::{
    ApiDef, DescriptorHeapDef, DeviceContext, Extents2D, Offset2D, Queue, QueueType,
    BACKBUFFER_COUNT,
};
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_input::keyboard::{KeyCode, KeyboardInput};

use lgn_tasks::ComputeTaskPool;
use lgn_tracing::span_scope;
pub use renderer::*;

mod render_context;
pub use render_context::*;

mod surface_renderer;

pub mod resources;
use resources::{
    DescriptorHeapManager, ModelManager, PersistentDescriptorSetManager, PipelineManager,
    TextureManager, TransientBufferAllocator, TransientCommandBufferAllocator,
    TransientCommandBufferManager,
};

pub mod components;

pub mod gpu_renderer;

pub mod picking;

pub mod debug_display;
pub mod egui;

pub(crate) mod lighting;
pub mod render_pass;

pub mod core;
pub mod features;
pub mod script;
pub mod shared;

mod renderdoc;

use crate::core::{
    GpuUploadManager, RenderCommandManager, RenderObjectsBuilder, RenderResourcesBuilder,
};

use crate::gpu_renderer::{ui_mesh_renderer, MeshRenderer};
use crate::renderdoc::RenderDocManager;
use crate::{
    components::{
        reflect_light_components, ManipulatorComponent, PickedComponent,
        RenderSurfaceCreatedForWindow, RenderSurfaceExtents, RenderSurfaces,
    },
    egui::{egui_plugin::EguiPlugin, Egui},
    lighting::LightingManager,
    picking::{ManipulatorManager, PickingManager, PickingPlugin},
    resources::MeshManager,
    RenderStage,
};
use lgn_app::{App, AppExit, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_math::{const_vec3, Vec3};
use lgn_transform::components::GlobalTransform;
use lgn_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};

use crate::debug_display::DebugDisplay;

use crate::resources::{
    ui_renderer_options, MaterialManager, MissingVisualTracker, RendererOptions, SamplerManager,
    SharedResourcesManager, TransientBufferManager, UnifiedStaticBuffer,
};

use crate::{
    components::{apply_camera_setups, camera_control, RenderSurface, VisualComponent},
    labels::CommandBufferLabel,
};

pub const UP_VECTOR: Vec3 = Vec3::Z;
pub const DOWN_VECTOR: Vec3 = const_vec3!([0_f32, 0_f32, -1_f32]);

#[derive(Clone)]
pub struct GraphicsQueue {
    queue: Arc<AtomicRefCell<Queue>>,
}

impl GraphicsQueue {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            queue: Arc::new(AtomicRefCell::new(
                device_context.create_queue(QueueType::Graphics),
            )),
        }
    }

    pub fn queue(&self) -> AtomicRef<'_, Queue> {
        self.queue.borrow()
    }

    pub fn queue_mut(&self) -> AtomicRefMut<'_, Queue> {
        self.queue.borrow_mut()
    }
}

#[derive(Default)]
pub struct RendererPlugin {}

impl Plugin for RendererPlugin {
    #[allow(unsafe_code)]
    fn build(&self, app: &mut App) {
        // TODO: Config resource? The renderer could be some kind of state machine reacting on some config changes?
        // TODO: refactor this with data pipeline resources
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_BRDF);
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_COMMON);
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_MESH);
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_FULLSCREEN_TRIANGLE);
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_TRANSFORM);
        EMBEDDED_FS.add_file(&gpu_renderer::SHADER_SHADER);

        const NUM_RENDER_FRAMES: u64 = BACKBUFFER_COUNT as u64 + 1;

        //
        // Init in dependency order
        //
        let gfx_api = GfxApiArc::new(ApiDef::default());
        let device_context = gfx_api.device_context();
        let graphics_queue = GraphicsQueue::new(device_context);
        let cgen_registry = Arc::new(cgen::initialize(device_context));
        let render_scope = RenderScope::new(NUM_RENDER_FRAMES, device_context);
        let upload_manager = GpuUploadManager::new();
        let static_buffer = UnifiedStaticBuffer::new(device_context, 64 * 1024 * 1024);
        let transient_buffer = TransientBufferManager::new(device_context, NUM_RENDER_FRAMES);
        let render_command_queue_pool = RenderCommandQueuePool::new();
        let render_command_manager = RenderCommandManager::new(&render_command_queue_pool);
        let mut render_commands = render_command_queue_pool.builder();
        let descriptor_heap_manager = DescriptorHeapManager::new(NUM_RENDER_FRAMES, device_context);
        let transient_commandbuffer_manager =
            TransientCommandBufferManager::new(NUM_RENDER_FRAMES, &graphics_queue);

        let mut pipeline_manager = PipelineManager::new(device_context);
        pipeline_manager.register_shader_families(&cgen_registry);

        let mut cgen_registry_list = CGenRegistryList::new();
        cgen_registry_list.push(cgen_registry);

        let mut persistent_descriptor_set_manager = PersistentDescriptorSetManager::new(
            device_context,
            &descriptor_heap_manager,
            NUM_RENDER_FRAMES,
        );

        let mut mesh_manager = MeshManager::new(static_buffer.allocator());
        mesh_manager.initialize_default_meshes(&mut render_commands);

        let texture_manager = TextureManager::new(device_context);

        let material_manager = MaterialManager::new(static_buffer.allocator());

        let sampler_manager =
            SamplerManager::new(device_context, &mut persistent_descriptor_set_manager);

        let shared_resources_manager = SharedResourcesManager::new(
            &mut render_commands,
            device_context,
            &mut persistent_descriptor_set_manager,
        );

        let render_layers_builder = RenderLayerBuilder::default();
        let render_layers = render_layers_builder
            .add_render_layer("DEPTH")
            .add_render_layer("OPAQUE")
            .add_render_layer("PICKING")
            .finalize();
        assert_eq!(
            render_layers.get_from_name("DEPTH").id(),
            RENDER_LAYER_DEPTH
        );
        assert_eq!(
            render_layers.get_from_name("OPAQUE").id(),
            RENDER_LAYER_OPAQUE
        );
        assert_eq!(
            render_layers.get_from_name("PICKING").id(),
            RENDER_LAYER_PICKING
        );

        let mesh_renderer =
            MeshRenderer::new(device_context, static_buffer.allocator(), &render_layers);
        let instance_manager = GpuInstanceManager::new(static_buffer.allocator());
        let manipulation_manager = ManipulatorManager::new();
        let picking_manager = PickingManager::new(4096);
        let model_manager = ModelManager::new(&mesh_manager, &material_manager);
        let missing_visuals_tracker = MissingVisualTracker::default();

        let light_manager = LightingManager::new();

        let renderdoc_manager = RenderDocManager::default();

        let render_objects = RenderObjectsBuilder::default()
            // Lights
            .add_primary_table::<RenderLight>()
            .add_secondary_table::<RenderLight, RenderLightTestData>()
            // Viewports
            .add_primary_table::<RenderViewport>()
            .add_secondary_table_with_handler::<RenderViewport, RenderViewportRendererData>(
                Box::new(RenderViewportPrivateDataHandler::new(
                    device_context.clone(),
                )),
            )
            // Visual
            .add_primary_table::<RenderVisual>()
            // Camera
            .add_primary_table::<RenderCamera>()
            // Done!
            .finalize();

        //
        // Add renderer stages first. It is needed for the plugins.
        //
        app.add_stage_after(
            CoreStage::PostUpdate,
            RenderStage::Resource,
            SystemStage::parallel(),
        );

        app.add_stage_after(
            RenderStage::Resource,
            RenderStage::Prepare,
            SystemStage::parallel(),
        );

        app.add_stage_after(
            RenderStage::Prepare,
            RenderStage::Render,
            SystemStage::parallel(),
        );

        //
        // Stage Startup
        //
        app.add_startup_system(init_manipulation_manager);
        app.add_startup_system(tmp_create_camera);

        //
        // RenderObjects
        //

        // Lights
        app.insert_resource(EcsToRenderLight::new(
            render_objects.primary_table_view::<RenderLight>(),
        ))
        .add_system_to_stage(RenderStage::Prepare, reflect_light_components);

        // Viewports
        app.insert_resource(EcsToRenderViewport::new(
            render_objects.primary_table_view::<RenderViewport>(),
        ))
        .add_system_to_stage(RenderStage::Prepare, reflect_viewports);

        // Model
        app.insert_resource(EcsToRenderVisual::new(
            render_objects.primary_table_view::<RenderVisual>(),
        ))
        .add_system_to_stage(RenderStage::Prepare, reflect_visual_components);

        // Camera
        app.insert_resource(EcsToRenderCamera::new(
            render_objects.primary_table_view::<RenderCamera>(),
        ))
        .add_system_to_stage(RenderStage::Prepare, reflect_camera_components);

        //
        // Resources
        //

        app.insert_resource(pipeline_manager)
            .insert_resource(manipulation_manager.clone())
            .insert_resource(cgen_registry_list)
            .insert_resource(RenderSurfaces::new())
            .insert_resource(DebugDisplay::default())
            .insert_resource(persistent_descriptor_set_manager)
            .insert_resource(shared_resources_manager)
            .insert_resource(texture_manager)
            .insert_resource(RendererOptions::default())
            .insert_resource(picking_manager.clone());

        // Init ecs
        TextureManager::init_ecs(app);
        MaterialManager::init_ecs(app);
        MeshRenderer::init_ecs(app);
        ModelManager::init_ecs(app);
        MissingVisualTracker::init_ecs(app);
        GpuInstanceManager::init_ecs(app);

        // Only Init AssetRegistry event handler if there's AssetRegistryEvent already registered
        if app
            .world
            .contains_resource::<Events<lgn_data_runtime::AssetRegistryEvent>>()
        {
            app.add_system_to_stage(RenderStage::Resource, asset_to_ecs::process_load_events);
        }

        // Plugins are optional
        app.add_plugin(EguiPlugin::default());
        app.add_plugin(PickingPlugin {});

        //
        // Events
        //
        app.add_event::<RenderSurfaceCreatedForWindow>();

        //
        // Stage PreUpdate
        //
        app.add_system_to_stage(CoreStage::PreUpdate, apply_camera_setups);

        //
        // Stage PostUpdate
        //

        // TODO (vbdd): CoreStage::PostUpdate is probably invalid. Anyway, this will change soon.

        app.add_system_to_stage(CoreStage::PostUpdate, on_window_created.exclusive_system());
        app.add_system_to_stage(CoreStage::PostUpdate, on_window_resized.exclusive_system());
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            on_window_close_requested.exclusive_system(),
        );

        //
        // Stage Prepare
        //
        app.add_system_to_stage(RenderStage::Prepare, ui_renderer_options);
        app.add_system_to_stage(RenderStage::Prepare, ui_mesh_renderer);
        app.add_system_to_stage(RenderStage::Prepare, tmp_debug_display_lights);
        app.add_system_to_stage(
            RenderStage::Prepare,
            camera_control.exclusive_system().at_start(),
        );

        //
        // Stage: Render
        //
        app.add_system_to_stage(
            RenderStage::Render,
            render_update.label(CommandBufferLabel::Generate),
        );

        //
        // Finalize
        //

        let render_features_builder = RenderFeaturesBuilder::new();
        let render_features = render_features_builder
            .insert(ModelFeature::new())
            .finalize();

        let render_graph_persistent_state = RenderGraphPersistentState::new();

        let render_resources_builder = RenderResourcesBuilder::new();
        let render_resources = render_resources_builder
            .insert(render_scope)
            .insert(gfx_api.clone())
            .insert(render_command_manager)
            .insert(upload_manager)
            .insert(static_buffer)
            .insert(transient_buffer)
            .insert(descriptor_heap_manager)
            .insert(transient_commandbuffer_manager)
            .insert(graphics_queue.clone())
            .insert(light_manager)
            .insert(renderdoc_manager)
            .insert(render_objects)
            .insert(instance_manager)
            .insert(mesh_renderer)
            .insert(manipulation_manager)
            .insert(picking_manager)
            .insert(model_manager)
            .insert(mesh_manager)
            .insert(material_manager)
            .insert(sampler_manager)
            .insert(missing_visuals_tracker)
            .insert(render_layers)
            .insert(render_features)
            .insert(render_graph_persistent_state)
            .insert(Herd::new())
            .finalize();

        app.add_system_to_stage(CoreStage::Last, on_app_exit);

        let renderer = Renderer::new(
            NUM_RENDER_FRAMES,
            render_command_queue_pool,
            render_resources,
            graphics_queue,
            gfx_api,
        );

        // This resource needs to be shutdown after all other resources
        app.insert_resource(renderer);
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn on_window_created(
    mut event_window_created: EventReader<'_, '_, WindowCreated>,
    window_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
    mut event_render_surface_created: ResMut<'_, Events<RenderSurfaceCreatedForWindow>>,
) {
    for ev in event_window_created.iter() {
        let wnd = window_list.get(ev.id).unwrap();
        let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());
        let mut render_surface = RenderSurface::new(wnd.id(), &renderer, extents);

        // TODO(jsg): Only one viewport for now.
        let viewport_offset = Offset2D { x: 0, y: 0 };
        let viewport_extents = Extents2D {
            width: extents.width(),
            height: extents.height(),
        };
        let viewport = Viewport::new(viewport_offset, viewport_extents);
        render_surface.add_viewport(viewport);

        render_surfaces.insert(render_surface);

        event_render_surface_created.send(RenderSurfaceCreatedForWindow { window_id: ev.id });
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_window_resized(
    mut ev_wnd_resized: EventReader<'_, '_, WindowResized>,
    wnd_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    let device_context = renderer.device_context();
    for ev in ev_wnd_resized.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();
        let render_surface = render_surfaces.try_get_from_window_id_mut(ev.id);
        if let Some(render_surface) = render_surface {
            render_surface.resize(
                device_context,
                RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height()),
            );
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_window_close_requested(
    mut ev_wnd_destroyed: EventReader<'_, '_, WindowCloseRequested>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
) {
    for ev in ev_wnd_destroyed.iter() {
        render_surfaces.remove_from_window_id(ev.id);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_manipulation_manager(
    commands: Commands<'_, '_>,
    mut manipulation_manager: ResMut<'_, ManipulatorManager>,
    picking_manager: Res<'_, PickingManager>,
) {
    manipulation_manager.initialize(commands, picking_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn on_app_exit(mut app_exit: EventReader<'_, '_, AppExit>, renderer: Res<'_, Renderer>) {
    // "Un dernier tour de crinque" to flush any remaining commands.
    if app_exit.iter().last().is_some() {
        renderer.graphics_queue().queue_mut().wait_for_queue_idle();
        renderer.device_context().free_gpu_memory();

        let mut render_objects = renderer.render_resources().get_mut::<RenderObjects>();
        render_objects.sync_update();
        render_objects.begin_frame();
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
fn render_update(
    task_pool: Res<'_, ComputeTaskPool>,
    resources: (
        Res<'_, Renderer>,
        ResMut<'_, PipelineManager>,
        ResMut<'_, PickingManager>,
        ResMut<'_, Egui>,
        ResMut<'_, DebugDisplay>,
        ResMut<'_, PersistentDescriptorSetManager>,
        ResMut<'_, RenderSurfaces>,
        EventReader<'_, '_, KeyboardInput>,
    ),
    queries: (
        Query<'_, '_, (&VisualComponent, &GlobalTransform), With<PickedComponent>>,
        Query<'_, '_, (&GlobalTransform, &ManipulatorComponent)>,
    ),
) {
    // resources
    let renderer = resources.0;
    let mut pipeline_manager = resources.1;
    let picking_manager = resources.2;
    let mut egui = resources.3;
    let mut debug_display = resources.4;
    let mut persistent_descriptor_set_manager = resources.5;
    let mut render_surfaces = resources.6;
    let mut keyboard_input_events = resources.7;

    // queries
    let q_picked_drawables = queries.0;
    let q_manipulator_drawables = queries.1;

    //
    // Simulation thread
    //

    {
        let mut render_commands = renderer.render_command_builder();

        for keyboard_input_event in keyboard_input_events.iter() {
            if let Some(key_code) = keyboard_input_event.key_code {
                if key_code == KeyCode::C && keyboard_input_event.state.is_pressed() {
                    render_commands.push(renderdoc::RenderDocCaptureCommand::default());
                }
            }
        }
    }

    let picked_drawables = q_picked_drawables
        .iter()
        .collect::<Vec<(&VisualComponent, &GlobalTransform)>>();
    let manipulator_drawables = q_manipulator_drawables
        .iter()
        .collect::<Vec<(&GlobalTransform, &ManipulatorComponent)>>();

    //
    // Wait for render thread
    //

    // todo

    //
    // Sync window (safe access to render resources)
    //

    let render_resources = renderer.render_resources().clone();

    {
        span_scope!("sync window");
        render_resources
            .get_mut::<RenderCommandManager>()
            .sync_update();

        render_resources.get_mut::<RenderObjects>().sync_update();
    }

    //
    // Egui (not thread safe as is)
    // we need to call the end_frame in the sync window I guess and transfer the data to the render thread
    //
    render_resources
        .get_mut::<LightingManager>()
        .debug_ui(egui.as_mut());
    crate::egui::egui_plugin::end_frame(&mut egui);

    // objectives: drop all resources/queries

    drop(renderer);
    drop(keyboard_input_events);

    //
    // Run render thread
    //

    task_pool.scope(|scope| {
        scope.spawn(async move {
            span_scope!("render_thread");

            let mut herd = render_resources.get_mut::<Herd>();
            let mut render_scope = render_resources.get_mut::<RenderScope>();
            let mut descriptor_heap_manager = render_resources.get_mut::<DescriptorHeapManager>();
            let device_context = render_resources.get::<GfxApiArc>().device_context().clone();
            let static_buffer = render_resources.get::<UnifiedStaticBuffer>();
            let mut transient_buffer = render_resources.get_mut::<TransientBufferManager>();
            let transient_commandbuffer_manager =
                render_resources.get::<TransientCommandBufferManager>();

            //
            // Begin frame (before commands)
            //

            herd.reset();
            render_scope.begin_frame();
            descriptor_heap_manager.begin_frame();

            device_context.free_gpu_memory();
            device_context.inc_current_cpu_frame();

            transient_buffer.begin_frame();
            transient_commandbuffer_manager.begin_frame();

            render_resources.get::<RenderObjects>().begin_frame();

            //
            // Update
            //
            render_resources
                .get_mut::<RenderCommandManager>()
                .apply(&render_resources);

            persistent_descriptor_set_manager.frame_update();
            pipeline_manager.frame_update(&device_context);

            let mut transient_commandbuffer_allocator =
                TransientCommandBufferAllocator::new(&transient_commandbuffer_manager);

            let graphics_queue = render_resources.get::<GraphicsQueue>();

            let mut transient_buffer_allocator =
                TransientBufferAllocator::new(&transient_buffer, 64 * 1024);

            render_resources.get_mut::<GpuUploadManager>().upload(
                &mut transient_commandbuffer_allocator,
                &mut transient_buffer_allocator,
                &graphics_queue,
            );

            //
            // Render
            //

            let mut renderdoc_manager = render_resources.get_mut::<RenderDocManager>();
            renderdoc_manager.start_frame_capture();

            {
                let descriptor_pool =
                    descriptor_heap_manager.acquire_descriptor_pool(default_descriptor_heap_size());

                let herd_member = herd.get();
                let bump = herd_member.as_bump();

                {
                    let render_layers = render_resources.get::<RenderLayers>();
                    let features = render_resources.get::<RenderFeatures>();

                    let render_context = RenderContext::new(
                        &device_context,
                        &graphics_queue,
                        &descriptor_pool,
                        &mut pipeline_manager,
                        &mut transient_commandbuffer_allocator,
                        &mut transient_buffer_allocator,
                        &static_buffer,
                        &herd,
                        bump,
                        &picking_manager,
                        &debug_display,
                        picked_drawables.as_slice(),
                        manipulator_drawables.as_slice(),
                        &egui,
                    );

                    SurfaceRenderer::render_surfaces(
                        render_scope.frame_idx(),
                        &mut render_surfaces,
                        &render_resources,
                        render_context,
                        &mut persistent_descriptor_set_manager,
                        &render_layers,
                        &features,
                    );
                }

                descriptor_heap_manager.release_descriptor_pool(descriptor_pool);
                drop(transient_buffer_allocator);
                drop(transient_commandbuffer_allocator);

                descriptor_heap_manager.end_frame();
                debug_display.end_frame();
                render_scope.end_frame(&graphics_queue);
                transient_buffer.end_frame();
                transient_commandbuffer_manager.end_frame();
            }

            renderdoc_manager.end_frame_capture();
        });
    });
}

fn default_descriptor_heap_size() -> DescriptorHeapDef {
    DescriptorHeapDef {
        max_descriptor_sets: 4096,
        sampler_count: 128,
        constant_buffer_count: 1024,
        buffer_count: 1024,
        rw_buffer_count: 1024,
        texture_count: 1024,
        rw_texture_count: 1024,
    }
}
