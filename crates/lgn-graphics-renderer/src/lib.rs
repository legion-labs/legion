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

use crate::core::RenderObjects;
use crate::lighting::{RenderLight, RenderLightTestData};
use std::sync::Arc;

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
#[allow(unused_imports, clippy::wildcard_imports)]
use cgen::*;

pub mod labels;

use gpu_renderer::GpuInstanceManager;

pub use labels::*;

mod asset_to_ecs;
mod renderer;
use lgn_embedded_fs::EMBEDDED_FS;
use lgn_graphics_api::{
    AddressMode, ApiDef, BufferViewDef, CompareOp, DescriptorHeapDef, DeviceContext, Extents3D,
    FilterType, Format, MemoryUsage, MipMapMode, Queue, QueueType, ResourceFlags, ResourceUsage,
    SamplerDef, TextureDef, TextureTiling,
};
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_input::keyboard::{KeyCode, KeyboardInput};
use lgn_math::Vec2;

use lgn_tasks::ComputeTaskPool;
use lgn_tracing::span_scope_named;
pub use renderer::*;

mod render_context;
pub use render_context::*;

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
pub mod shared;

mod renderdoc;

use crate::core::{
    GpuUploadManager, RenderCommandBuilder, RenderCommandManager, RenderCommandQueuePool,
    RenderManagers, RenderObjectsBuilder, RenderResourcesBuilder,
};
use crate::gpu_renderer::{ui_mesh_renderer, MeshRenderer};
use crate::render_pass::TmpRenderPass;
use crate::renderdoc::RenderDocManager;
use crate::{
    components::{
        reflect_light_components, ManipulatorComponent, PickedComponent,
        RenderSurfaceCreatedForWindow, RenderSurfaceExtents, RenderSurfaces,
    },
    core::render_graph::{
        AlphaBlendedLayerPass, Config, DepthLayerPass, GpuCullingPass, LightingPass,
        OpaqueLayerPass, PostProcessPass, RenderScript, RenderView, SSAOPass, UiPass,
    },
    egui::{egui_plugin::EguiPlugin, Egui},
    lighting::LightingManager,
    picking::{ManipulatorManager, PickingManager, PickingPlugin},
    resources::MeshManager,
    RenderStage,
};
use lgn_app::{App, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_math::{const_vec3, Vec3};
use lgn_transform::components::GlobalTransform;
use lgn_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};

use crate::debug_display::DebugDisplay;

use crate::resources::{
    ui_renderer_options, MaterialManager, MissingVisualTracker, RendererOptions,
    SharedResourcesManager, TransientBufferManager, UnifiedStaticBuffer,
};

use crate::{
    components::{
        apply_camera_setups, camera_control, create_camera, CameraComponent, LightComponent,
        RenderSurface, VisualComponent,
    },
    labels::CommandBufferLabel,
};

pub const UP_VECTOR: Vec3 = Vec3::Y;
pub const DOWN_VECTOR: Vec3 = const_vec3!([0_f32, -1_f32, 0_f32]);

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
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_TRANSFORM);
        EMBEDDED_FS.add_file(&gpu_renderer::SHADER_SHADER);

        const NUM_RENDER_FRAMES: u64 = 2;

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
        let render_command_manager = RenderCommandManager::new();
        let render_command_queue_pool = RenderCommandQueuePool::new();
        let mut render_commands = RenderCommandBuilder::new(&render_command_queue_pool);
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

        let shared_resources_manager = SharedResourcesManager::new(
            &mut render_commands,
            device_context,
            &mut persistent_descriptor_set_manager,
        );

        let mesh_renderer = MeshRenderer::new(device_context, static_buffer.allocator());

        let light_manager = LightingManager::new(device_context);

        let renderdoc_manager = RenderDocManager::default();

        let render_objects = RenderObjectsBuilder::default()
            .add_primary_table::<RenderLight>()
            .add_secondary_table::<RenderLight, RenderLightTestData>()
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
        app.add_startup_system(create_camera);

        //
        // Resources
        //
        app.insert_resource(pipeline_manager);
        app.insert_resource(ManipulatorManager::new());
        app.insert_resource(cgen_registry_list);
        app.insert_resource(RenderSurfaces::new());
        app.insert_resource(ModelManager::new(&mesh_manager, &material_manager));
        app.insert_resource(mesh_manager);
        app.insert_resource(DebugDisplay::default());
        app.insert_resource(GpuInstanceManager::new(static_buffer.allocator()));
        app.insert_resource(MissingVisualTracker::default());
        app.insert_resource(persistent_descriptor_set_manager);
        app.insert_resource(shared_resources_manager);
        app.insert_resource(texture_manager);
        app.insert_resource(material_manager);
        app.insert_resource(mesh_renderer);
        app.insert_resource(RendererOptions::default());

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
        app.add_system_to_stage(RenderStage::Prepare, reflect_light_components);
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
            .finalize();

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
    mut commands: Commands<'_, '_>,
    mut event_window_created: EventReader<'_, '_, WindowCreated>,
    window_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    pipeline_manager: Res<'_, PipelineManager>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
    mut event_render_surface_created: ResMut<'_, Events<RenderSurfaceCreatedForWindow>>,
) {
    for ev in event_window_created.iter() {
        let wnd = window_list.get(ev.id).unwrap();
        let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());
        let render_surface = RenderSurface::new(&renderer, &pipeline_manager, extents);

        render_surfaces.insert(ev.id, render_surface.id());

        event_render_surface_created.send(RenderSurfaceCreatedForWindow {
            window_id: ev.id,
            render_surface_id: render_surface.id(),
        });

        commands.spawn().insert(render_surface);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_window_resized(
    mut ev_wnd_resized: EventReader<'_, '_, WindowResized>,
    wnd_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    render_surfaces: Res<'_, RenderSurfaces>,
    pipeline_manager: Res<'_, PipelineManager>,
) {
    let device_context = renderer.device_context();
    for ev in ev_wnd_resized.iter() {
        let render_surface_id = render_surfaces.get_from_window_id(ev.id);
        if let Some(render_surface_id) = render_surface_id {
            let render_surface = q_render_surfaces
                .iter_mut()
                .find(|x| x.id() == *render_surface_id);
            if let Some(mut render_surface) = render_surface {
                let wnd = wnd_list.get(ev.id).unwrap();
                render_surface.resize(
                    &device_context,
                    RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height()),
                    &pipeline_manager,
                );
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
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
            render_surfaces.remove(ev.id);
        }
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

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity,
    unsafe_code
)]
fn render_update(
    task_pool: Res<'_, ComputeTaskPool>,
    resources: (
        ResMut<'_, Renderer>,
        ResMut<'_, PipelineManager>,
        ResMut<'_, MeshRenderer>,
        Res<'_, MeshManager>,
        Res<'_, PickingManager>,
        Res<'_, GpuInstanceManager>,
        ResMut<'_, Egui>,
        ResMut<'_, DebugDisplay>,
        ResMut<'_, PersistentDescriptorSetManager>,
        Res<'_, ModelManager>,
        EventReader<'_, '_, KeyboardInput>,
    ),
    queries: (
        Query<'_, '_, &mut RenderSurface>,
        Query<'_, '_, (&VisualComponent, &GlobalTransform), With<PickedComponent>>,
        Query<'_, '_, (&GlobalTransform, &ManipulatorComponent)>,
        Query<'_, '_, (&LightComponent, &GlobalTransform)>,
        Query<'_, '_, &CameraComponent>,
    ),
) {
    // resources
    let mut renderer = resources.0;
    let mut pipeline_manager = resources.1;
    let mut mesh_renderer = resources.2;
    let mesh_manager = resources.3;
    let picking_manager = resources.4;
    let instance_manager = resources.5;
    let mut egui = resources.6;
    let mut debug_display = resources.7;
    let mut persistent_descriptor_set_manager = resources.8;
    let model_manager = resources.9;
    let mut keyboard_input_events = resources.10;

    // queries
    let mut q_render_surfaces = queries.0;
    let q_picked_drawables = queries.1;
    let q_manipulator_drawables = queries.2;
    let q_lights = queries.3;
    let q_cameras = queries.4;

    //
    // Simulation thread
    //

    let mut render_commands = renderer.render_command_builder();

    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key_code) = keyboard_input_event.key_code {
            if key_code == KeyCode::C && keyboard_input_event.state.is_pressed() {
                render_commands.push(renderdoc::RenderDocCaptureCommand::default());
            }
        }
    }

    let picked_drawables = q_picked_drawables
        .iter()
        .collect::<Vec<(&VisualComponent, &GlobalTransform)>>();
    let manipulator_drawables = q_manipulator_drawables
        .iter()
        .collect::<Vec<(&GlobalTransform, &ManipulatorComponent)>>();
    let lights = q_lights
        .iter()
        .collect::<Vec<(&LightComponent, &GlobalTransform)>>();

    //
    // Wait for render thread
    //

    // todo

    //
    // Sync window (safe access to render resources)
    //

    let render_resources = renderer.render_resources().clone();

    render_resources
        .get_mut::<RenderCommandManager>()
        .sync_update(renderer.render_command_queue_pool());
    // render_resources.get_mut::<RenderObjectSet<RenderLight>>().sync_update( &mut render_resources.get_mut::<RenderObjectSetAllocator<RenderLight>>()  );
    render_resources.get_mut::<RenderObjects>().sync_update();

    // objectives: drop all resources/queries

    drop(renderer);
    drop(keyboard_input_events);

    //
    // Run render thread
    //

    task_pool.scope( |scope| {
        scope.spawn(  async move {
        span_scope_named!("render_thread");       

        let q_cameras = q_cameras.iter().collect::<Vec<&CameraComponent>>();
        let default_camera = CameraComponent::default();
        let camera_component = if !q_cameras.is_empty() {
            q_cameras[0]
        } else {
            &default_camera
        };

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

        let render_objects = render_resources.get::<RenderObjects>();
        render_resources.get::<LightingManager>().frame_update(&render_objects);
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

        crate::egui::egui_plugin::end_frame(&mut egui);

        let mut renderdoc_manager = render_resources.get_mut::<RenderDocManager>();
        renderdoc_manager.start_frame_capture();

        {
            let descriptor_pool =
            descriptor_heap_manager.acquire_descriptor_pool(default_descriptor_heap_size());

        let mut render_context = RenderContext::new(
                &device_context,
            &graphics_queue,
            &descriptor_pool,
            &pipeline_manager,
            &mut transient_commandbuffer_allocator,
            &mut transient_buffer_allocator,
            &static_buffer,
        );

        // Persistent descriptor set
        {
            let descriptor_set = persistent_descriptor_set_manager.descriptor_set();
            render_context
                .set_persistent_descriptor_set(descriptor_set.layout(), *descriptor_set.handle());
        }

        // Frame descriptor set
        {
            let mut frame_descriptor_set = cgen::descriptor_set::FrameDescriptorSet::default();

                render_resources.get::<LightingManager>().per_frame_render(
                render_context.transient_buffer_allocator,
                &mut frame_descriptor_set,
            );

            let static_buffer_ro_view = static_buffer.read_only_view();
            frame_descriptor_set.set_static_buffer(static_buffer_ro_view);

            let va_table_address_buffer = instance_manager.structured_buffer_view();
            frame_descriptor_set.set_va_table_address_buffer(va_table_address_buffer);

            let sampler_def = SamplerDef {
                min_filter: FilterType::Linear,
                mag_filter: FilterType::Linear,
                mip_map_mode: MipMapMode::Linear,
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mip_lod_bias: 0.0,
                max_anisotropy: 1.0,
                compare_op: CompareOp::LessOrEqual,
            };
            let material_sampler = render_context.device_context.create_sampler(sampler_def);
            frame_descriptor_set.set_material_sampler(&material_sampler);

            let frame_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::FrameDescriptorSet::descriptor_set_layout(),
                frame_descriptor_set.descriptor_refs(),
            );

            render_context.set_frame_descriptor_set(
                cgen::descriptor_set::FrameDescriptorSet::descriptor_set_layout(),
                frame_descriptor_set_handle,
            );
        }

        // For each surface/view, we have to execute the render graph
        for mut render_surface in q_render_surfaces.iter_mut() {
            // View descriptor set
            {
                let mut screen_rect = picking_manager.screen_rect();
                if screen_rect.x == 0.0 || screen_rect.y == 0.0 {
                    screen_rect = Vec2::new(
                        render_surface.extents().width() as f32,
                        render_surface.extents().height() as f32,
                    );
                }

                let cursor_pos = picking_manager.current_cursor_pos();

                let view_data = camera_component.tmp_build_view_data(
                    render_surface.extents().width() as f32,
                    render_surface.extents().height() as f32,
                    screen_rect.x,
                    screen_rect.y,
                    cursor_pos.x,
                    cursor_pos.y,
                );

                let sub_allocation = render_context
                    .transient_buffer_allocator
                    .copy_data(&view_data, ResourceUsage::AS_CONST_BUFFER);

                let const_buffer_view = sub_allocation
                    .to_buffer_view(BufferViewDef::as_const_buffer_typed::<cgen_type::ViewData>());

                let mut view_descriptor_set = cgen::descriptor_set::ViewDescriptorSet::default();
                view_descriptor_set.set_view_data(const_buffer_view);

                view_descriptor_set
                    .set_hzb_texture(render_surface.get_hzb_surface().hzb_srv_view());

                let view_descriptor_set_handle = render_context.write_descriptor_set(
                    cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                    view_descriptor_set.descriptor_refs(),
                );

                render_context.set_view_descriptor_set(
                    cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                    view_descriptor_set_handle,
                );
            }

            let mut cmd_buffer_handle = render_context.transient_commandbuffer_allocator.acquire();
            let cmd_buffer = cmd_buffer_handle.as_mut();

            cmd_buffer.begin();

            mesh_renderer.gen_occlusion_and_cull(
                &mut render_context,
                cmd_buffer,
                &mut render_surface,
                &instance_manager,
            );

            cmd_buffer.cmd_bind_index_buffer(static_buffer.index_buffer_binding());
            cmd_buffer.cmd_bind_vertex_buffer(0, instance_manager.vertex_buffer_binding());

            let picking_pass = render_surface.picking_renderpass();
            let mut picking_pass = picking_pass.write();
            picking_pass.render(
                &picking_manager,
                &render_context,
                cmd_buffer,
                render_surface.as_mut(),
                &instance_manager,
                    manipulator_drawables.as_slice(),
                    lights.as_slice(),
                &mesh_manager,
                camera_component,
                &mesh_renderer,
            );

            // TmpRenderPass::render(
            //     &render_context,
            //     cmd_buffer,
            //     render_surface.as_mut(),
            //     &mesh_renderer,
            // );

            let debug_renderpass = render_surface.debug_renderpass();
            let debug_renderpass = debug_renderpass.write();
            debug_renderpass.render(
                &render_context,
                cmd_buffer,
                render_surface.as_mut(),
                    picked_drawables.as_slice(),
                    manipulator_drawables.as_slice(),
                camera_component,
                &mesh_manager,
                &model_manager,
                &debug_display,
            );

            if egui.is_enabled() {
                let egui_pass = render_surface.egui_renderpass();
                let mut egui_pass = egui_pass.write();
                egui_pass.update_font_texture(&render_context, cmd_buffer, egui.ctx());
                egui_pass.render(
                    &mut render_context,
                    cmd_buffer,
                    render_surface.as_mut(),
                    &egui,
                );
            }

            cmd_buffer.end();

            let test_render_graph = false;
            if test_render_graph {
                render_context
                    .graphics_queue
                    .queue_mut()
                    .submit(&[cmd_buffer], &[], &[], None);

                render_context
                    .transient_commandbuffer_allocator
                    .release(cmd_buffer_handle);

                let mut cmd_buffer_handle =
                    render_context.transient_commandbuffer_allocator.acquire();
                let cmd_buffer = cmd_buffer_handle.as_mut();

                cmd_buffer.begin();

                //****************************************************************
                cmd_buffer.with_label("RenderGraph", |cmd_buffer| {

                    let gpu_culling_pass = GpuCullingPass;
                    let depth_layer_pass = DepthLayerPass;
                    let opaque_layer_pass = OpaqueLayerPass;
                    let ssao_pass = SSAOPass;
                    let alphablended_layer_pass = AlphaBlendedLayerPass;
                    let postprocess_pass = PostProcessPass;
                    let lighting_pass = LightingPass;
                    let ui_pass = UiPass;

                    let view_desc = TextureDef {
                        extents: Extents3D {
                            width: 1920,
                            height: 1080,
                            depth: 1,
                        },
                        array_length: 1,
                        mip_count: 1,
                        format: Format::R8G8B8A8_UNORM,
                        usage_flags: ResourceUsage::AS_RENDER_TARGET | ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_UNORDERED_ACCESS| ResourceUsage::AS_TRANSFERABLE,
                        resource_flags: ResourceFlags::empty(),
                        memory_usage: MemoryUsage::GpuOnly,
                        tiling: TextureTiling::Optimal,
                    };
                    let view_target = render_context.device_context.create_texture(view_desc, "ViewBuffer");
                    let view = RenderView {
                        target: view_target,
                    };

                    let depth_desc = TextureDef {
                        extents: view.target.definition().extents,
                        array_length: 1,
                        mip_count: 1,
                        format: Format::D24_UNORM_S8_UINT,
                        usage_flags: ResourceUsage::AS_DEPTH_STENCIL | ResourceUsage::AS_SHADER_RESOURCE,
                        resource_flags: ResourceFlags::empty(),
                        memory_usage: MemoryUsage::GpuOnly,
                        tiling: TextureTiling::Optimal,
                    };
                    let prev_depth = render_context.device_context.create_texture(depth_desc, "PrevDepthBuffer");

                    let mut render_script = RenderScript {
                        gpu_culling_pass,
                        depth_layer_pass,
                        opaque_layer_pass,
                        ssao_pass,
                        alphablended_layer_pass,
                        postprocess_pass,
                        lighting_pass,
                        ui_pass,
                        prev_depth,
                    };

                    let config = Config::default();

                    match render_script.build_render_graph(&view, &config, render_context.pipeline_manager, render_context.device_context) {
                        Ok(render_graph) => {
                            // Print out the render graph
                            println!("{}", render_graph);
                            println!("\n\n");

                            let mut render_graph_context = render_graph.compile();

                            println!("\n\n");

                            let render_managers = RenderManagers {
                                mesh_renderer: &mesh_renderer,
                                instance_manager: &instance_manager,
                            };

                            // Execute it
                            println!("*****************************************************************************");
                            println!("Frame {}", render_scope.frame_idx());
                            render_graph.execute(
                                &mut render_graph_context,
                                &render_resources,
                                &render_managers,
                                &mut render_context,
                                cmd_buffer,
                            );
                        }
                        Err(error) => {
                            println!("{}", error);
                        }
                    }

                });

                //****************************************************************

                cmd_buffer.end();

                // queue
                let present_sema = render_surface.acquire();
                {
                    render_context.graphics_queue.queue_mut().submit(
                        &[cmd_buffer],
                        &[present_sema],
                        &[],
                        None,
                    );

                    render_surface.present(&mut render_context);
                }

                render_context
                    .transient_commandbuffer_allocator
                    .release(cmd_buffer_handle);
            } else {
                // queue
                let present_sema = render_surface.acquire();
                {
                    render_context.graphics_queue.queue_mut().submit(
                        &[cmd_buffer],
                        &[present_sema],
                        &[],
                        None,
                    );

                    render_surface.present(&mut render_context);
                }

                render_context
                    .transient_commandbuffer_allocator
                    .release(cmd_buffer_handle);
            }
        }

        descriptor_heap_manager.release_descriptor_pool(descriptor_pool);
        drop(transient_buffer_allocator);
        drop(transient_commandbuffer_allocator);

        descriptor_heap_manager.end_frame();
        debug_display.end_frame();
        render_scope.end_frame(&graphics_queue);
        transient_buffer.end_frame();
        transient_commandbuffer_manager.end_frame();
        mesh_renderer.end_frame();

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
