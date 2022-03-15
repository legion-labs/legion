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

use std::sync::Arc;

#[allow(unused_imports)]
use cgen::*;

mod labels;
use components::MaterialComponent;
use gpu_renderer::{GpuInstanceEvent, GpuInstanceManager, RenderElement};
pub use labels::*;

mod renderer;
use lgn_embedded_fs::EMBEDDED_FS;
use lgn_graphics_api::{AddressMode, CompareOp, FilterType, MipMapMode, ResourceUsage, SamplerDef};
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_math::{Vec2, Vec4};
pub use renderer::*;

mod render_context;
pub use render_context::*;

pub mod resources;
use resources::{
    DescriptorHeapManager, GpuDataPlugin, GpuEntityColorManager, GpuEntityTransformManager,
    GpuPickingDataManager, MaterialManager, ModelManager, PersistentDescriptorSetManager,
    PipelineManager, TextureManager,
};

pub mod components;

pub mod gpu_renderer;

pub mod picking;

pub mod debug_display;
pub mod egui;

pub mod hl_gfx_api;

pub(crate) mod lighting;
pub(crate) mod render_pass;

use crate::gpu_renderer::MeshRenderer;
use crate::render_pass::TmpRenderPass;
use crate::{
    components::{
        debug_display_lights, ui_lights, update_lights, ManipulatorComponent, PickedComponent,
        RenderSurfaceCreatedForWindow, RenderSurfaceExtents, RenderSurfaces,
    },
    egui::egui_plugin::{Egui, EguiPlugin},
    gpu_renderer::GpuInstanceVas,
    lighting::LightingManager,
    picking::{ManipulatorManager, PickingIdContext, PickingManager, PickingPlugin},
    resources::MeshManager,
    RenderStage,
};
use lgn_app::{App, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_math::{const_vec3, Vec3};
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;
use lgn_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};

use crate::debug_display::DebugDisplay;

use crate::resources::{
    ui_renderer_options, MissingVisualTracker, RendererOptions, SharedResourcesManager,
    UniformGPUDataUpdater,
};

use crate::{
    components::{
        camera_control, create_camera, CameraComponent, LightComponent, RenderSurface,
        VisualComponent,
    },
    labels::CommandBufferLabel,
};

pub const UP_VECTOR: Vec3 = Vec3::Y;
pub const DOWN_VECTOR: Vec3 = const_vec3!([0_f32, -1_f32, 0_f32]);

#[derive(Default)]
pub struct RendererPlugin {}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        // TODO: Config resource? The renderer could be some kind of state machine reacting on some config changes?
        // TODO: refactor this with data pipeline resources
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_BRDF);
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_COMMON);
        EMBEDDED_FS.add_file(&gpu_renderer::INCLUDE_MESH);
        EMBEDDED_FS.add_file(&gpu_renderer::SHADER_SHADER);

        const NUM_RENDER_FRAMES: usize = 2;

        //
        // Init in dependency order
        //
        let renderer = Renderer::new(NUM_RENDER_FRAMES);
        let cgen_registry = Arc::new(cgen::initialize(renderer.device_context()));
        let descriptor_heap_manager =
            DescriptorHeapManager::new(NUM_RENDER_FRAMES, renderer.device_context());
        let mut pipeline_manager = PipelineManager::new(renderer.device_context());
        pipeline_manager.register_shader_families(&cgen_registry);
        let mut cgen_registry_list = CGenRegistryList::new();
        cgen_registry_list.push(cgen_registry);
        let mut persistent_descriptor_set_manager = PersistentDescriptorSetManager::new(
            renderer.device_context(),
            &descriptor_heap_manager,
        );
        let texture_manager = TextureManager::new(renderer.device_context());

        let shared_resources_manager =
            SharedResourcesManager::new(&renderer, &mut persistent_descriptor_set_manager);

        let mesh_renderer = MeshRenderer::new(renderer.static_buffer_allocator());
        let debug_display = DebugDisplay::default();

        //
        // Add renderer stages first. It is needed for the plugins.
        //
        app.add_stage_after(
            CoreStage::PostUpdate,
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
        app.insert_resource(ModelManager::new());
        app.insert_resource(MeshManager::new(&renderer));
        app.insert_resource(debug_display);
        app.insert_resource(LightingManager::default());
        app.insert_resource(GpuInstanceManager::new(renderer.static_buffer_allocator()));
        app.insert_resource(MissingVisualTracker::default());
        app.insert_resource(descriptor_heap_manager);
        app.insert_resource(persistent_descriptor_set_manager);
        app.insert_resource(shared_resources_manager);
        app.insert_resource(texture_manager);
        app.insert_resource(mesh_renderer);
        app.init_resource::<RendererOptions>();

        // Init ecs
        TextureManager::init_ecs(app);
        MeshRenderer::init_ecs(app);
        ModelManager::init_ecs(app);
        MissingVisualTracker::init_ecs(app);

        // todo: convert?
        app.add_plugin(GpuDataPlugin::default());

        // Plugins are optionnal
        app.add_plugin(EguiPlugin::new());
        app.add_plugin(PickingPlugin {});

        // This resource needs to be shutdown after all other resources
        app.insert_resource(renderer);

        //
        // Events
        //
        app.add_event::<RenderSurfaceCreatedForWindow>();

        //
        // Stage PreUpdate
        //
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update);

        //
        // Stage PostUpdate
        //
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
        app.add_system_to_stage(RenderStage::Prepare, ui_lights);
        app.add_system_to_stage(RenderStage::Prepare, debug_display_lights);
        app.add_system_to_stage(RenderStage::Prepare, resources::debug_bounding_spheres);
        app.add_system_to_stage(RenderStage::Prepare, update_gpu_instances);
        app.add_system_to_stage(RenderStage::Prepare, update_lights);
        app.add_system_to_stage(
            RenderStage::Prepare,
            camera_control.exclusive_system().at_start(),
        );
        app.add_system_to_stage(RenderStage::Prepare, prepare_shaders);

        //
        // Stage: Render
        //
        app.add_system_to_stage(
            RenderStage::Render,
            render_begin.exclusive_system().at_start(),
        );

        app.add_system_to_stage(
            RenderStage::Render,
            render_update.label(CommandBufferLabel::Generate),
        );

        app.add_system_to_stage(RenderStage::Render, render_end.exclusive_system().at_end());
    }
}

#[allow(clippy::needless_pass_by_value)]
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
    for ev in ev_wnd_resized.iter() {
        let render_surface_id = render_surfaces.get_from_window_id(ev.id);
        if let Some(render_surface_id) = render_surface_id {
            let render_surface = q_render_surfaces
                .iter_mut()
                .find(|x| x.id() == *render_surface_id);
            if let Some(mut render_surface) = render_surface {
                let wnd = wnd_list.get(ev.id).unwrap();
                render_surface.resize(
                    renderer.device_context(),
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
        }
        render_surfaces.remove(ev.id);
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
fn render_pre_update(
    mut renderer: ResMut<'_, Renderer>,
    mut descriptor_heap_manager: ResMut<'_, DescriptorHeapManager>,
) {
    renderer.begin_frame();
    descriptor_heap_manager.begin_frame();
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn update_gpu_instances(
    renderer: Res<'_, Renderer>,
    picking_manager: Res<'_, PickingManager>,
    mut picking_data_manager: ResMut<'_, GpuPickingDataManager>,
    mut instance_manager: ResMut<'_, GpuInstanceManager>,
    model_manager: Res<'_, ModelManager>,
    mesh_manager: Res<'_, MeshManager>,
    material_manager: Res<'_, MaterialManager>,
    color_manager: Res<'_, GpuEntityColorManager>,
    transform_manager: Res<'_, GpuEntityTransformManager>,
    mut event_writer: EventWriter<'_, '_, GpuInstanceEvent>,
    instance_query: Query<
        '_,
        '_,
        (Entity, &VisualComponent, Option<&MaterialComponent>),
        (Changed<VisualComponent>, Without<ManipulatorComponent>),
    >,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let mut picking_context = PickingIdContext::new(&picking_manager);

    for (entity, _, _) in instance_query.iter() {
        picking_data_manager.remove_gpu_data(&entity);
        if let Some(removed_ids) = instance_manager.remove_gpu_instance(entity) {
            event_writer.send(GpuInstanceEvent::Removed(removed_ids));
        }
    }

    for (entity, visual, mat_component) in instance_query.iter() {
        let color: (f32, f32, f32, f32) = (
            f32::from(visual.color.r) / 255.0f32,
            f32::from(visual.color.g) / 255.0f32,
            f32::from(visual.color.b) / 255.0f32,
            f32::from(visual.color.a) / 255.0f32,
        );
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());

        instance_color.set_color_blend(visual.color_blend.into());

        color_manager.update_gpu_data(&entity, 0, &instance_color, &mut updater);

        let mut material_key = None;
        if let Some(material) = mat_component {
            material_key = Some(material.material_id);
        }

        picking_data_manager.alloc_gpu_data(entity, renderer.static_buffer_allocator());

        let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
        picking_data.set_picking_id(picking_context.aquire_picking_id(entity).into());
        picking_data_manager.update_gpu_data(&entity, 0, &picking_data, &mut updater);

        let (model_meta_data, ready) = model_manager.get_model_meta_data(visual);
        if !ready {
            if let Some(model_resource_id) = &visual.model_resource_id {
                missing_visuals_tracker.add_entity(*model_resource_id, entity);
            }
        }

        let mut added_instances = Vec::with_capacity(model_meta_data.meshes.len());
        for mesh in &model_meta_data.meshes {
            let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);

            let instance_vas = GpuInstanceVas {
                submesh_va: mesh_meta_data.mesh_description_offset,
                material_va: if mesh.material_id != u32::MAX {
                    mesh.material_id
                } else {
                    material_manager.gpu_data().va_for_index(material_key, 0) as u32
                },
                color_va: color_manager.va_for_index(Some(entity), 0) as u32,
                transform_va: transform_manager.va_for_index(Some(entity), 0) as u32,
                picking_data_va: picking_data_manager.va_for_index(Some(entity), 0) as u32,
            };

            let gpu_instance_id = instance_manager.add_gpu_instance(
                entity,
                renderer.static_buffer_allocator(),
                &mut updater,
                &instance_vas,
            );

            added_instances.push((
                if mesh.material_id != u32::MAX {
                    mesh.material_index
                } else {
                    material_manager.gpu_data().id_for_index(material_key, 0)
                },
                RenderElement::new(gpu_instance_id, mesh.mesh_id as u32, &mesh_manager),
            ));
        }
        event_writer.send(GpuInstanceEvent::Added(added_instances));
    }

    renderer.add_update_job_block(updater.job_blocks());
}

fn prepare_shaders(mut pipeline_manager: ResMut<'_, PipelineManager>) {
    pipeline_manager.update();
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
fn render_begin(mut egui_manager: ResMut<'_, Egui>) {
    crate::egui::egui_plugin::end_frame(&mut egui_manager);
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
fn render_update(
    resources: (
        Res<'_, Renderer>,
        Res<'_, TextureManager>, // unused
        Res<'_, PipelineManager>,
        Res<'_, MeshRenderer>,
        Res<'_, MeshManager>,
        Res<'_, PickingManager>,
        Res<'_, GpuInstanceManager>,
        Res<'_, Egui>,
        Res<'_, DebugDisplay>,
        Res<'_, LightingManager>,
        Res<'_, DescriptorHeapManager>,
        Res<'_, PersistentDescriptorSetManager>,
        Res<'_, ModelManager>,
    ),
    queries: (
        Query<'_, '_, &mut RenderSurface>,
        Query<
            '_,
            '_,
            (&VisualComponent, &GlobalTransform),
            (With<PickedComponent>, Without<ManipulatorComponent>),
        >,
        Query<'_, '_, (&VisualComponent, &GlobalTransform, &ManipulatorComponent)>,
        Query<'_, '_, (&LightComponent, &GlobalTransform)>,
        Query<'_, '_, &CameraComponent>,
    ),
) {
    // resources
    let renderer = resources.0;
    // let bindless_texture_manager = resources.1;
    let pipeline_manager = resources.2;
    let mesh_renderer = resources.3;
    let mesh_manager = resources.4;
    let picking_manager = resources.5;
    let instance_manager = resources.6;
    let egui = resources.7;
    let debug_display = resources.8;
    let lighting_manager = resources.9;
    let descriptor_heap_manager = resources.10;
    let persistent_descriptor_set_manager = resources.11;
    let model_manager = resources.12;

    // queries
    let mut q_render_surfaces = queries.0;
    let q_picked_drawables = queries.1;
    let q_manipulator_drawables = queries.2;
    let q_lights = queries.3;
    let q_cameras = queries.4;

    // start
    let mut render_context =
        RenderContext::new(&renderer, &descriptor_heap_manager, &pipeline_manager);
    let q_picked_drawables = q_picked_drawables
        .iter()
        .collect::<Vec<(&VisualComponent, &GlobalTransform)>>();
    let q_manipulator_drawables = q_manipulator_drawables.iter().collect::<Vec<(
        &VisualComponent,
        &GlobalTransform,
        &ManipulatorComponent,
    )>>();
    let q_lights = q_lights
        .iter()
        .collect::<Vec<(&LightComponent, &GlobalTransform)>>();

    let q_cameras = q_cameras.iter().collect::<Vec<&CameraComponent>>();
    let default_camera = CameraComponent::default();
    let camera_component = if !q_cameras.is_empty() {
        q_cameras[0]
    } else {
        &default_camera
    };

    renderer.flush_update_jobs(&render_context);

    // Persistent descriptor set
    {
        let descriptor_set = persistent_descriptor_set_manager.descriptor_set();
        render_context
            .set_persistent_descriptor_set(descriptor_set.layout(), *descriptor_set.handle());
    }

    // Frame descriptor set
    {
        let mut frame_descriptor_set = cgen::descriptor_set::FrameDescriptorSet::default();

        let lighting_manager_view = render_context
            .transient_buffer_allocator()
            .copy_data(&lighting_manager.gpu_data(), ResourceUsage::AS_CONST_BUFFER)
            .const_buffer_view();
        frame_descriptor_set.set_lighting_data(&lighting_manager_view);

        let omni_lights_buffer_view = renderer.omnidirectional_lights_data_structured_buffer_view();
        frame_descriptor_set.set_omni_directional_lights(&omni_lights_buffer_view);

        let directionnal_lights_buffer_view =
            renderer.directional_lights_data_structured_buffer_view();
        frame_descriptor_set.set_directional_lights(&directionnal_lights_buffer_view);

        let spot_lights_buffer_view = renderer.spotlights_data_structured_buffer_view();
        frame_descriptor_set.set_spot_lights(&spot_lights_buffer_view);

        let static_buffer_ro_view = renderer.static_buffer_ro_view();
        frame_descriptor_set.set_static_buffer(&static_buffer_ro_view);

        let va_table_address_buffer =
            instance_manager.structured_buffer_view(std::mem::size_of::<u32>() as u64, true);
        frame_descriptor_set.set_va_table_address_buffer(&va_table_address_buffer);

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
        let material_sampler = renderer.device_context().create_sampler(&sampler_def);
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
                .transient_buffer_allocator()
                .copy_data(&view_data, ResourceUsage::AS_CONST_BUFFER);

            let const_buffer_view = sub_allocation.const_buffer_view();

            let mut view_descriptor_set = cgen::descriptor_set::ViewDescriptorSet::default();
            view_descriptor_set.set_view_data(&const_buffer_view);

            view_descriptor_set.set_hzb_texture(render_surface.get_hzb_surface().hzb_srv_view());

            let view_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                view_descriptor_set.descriptor_refs(),
            );

            render_context.set_view_descriptor_set(
                cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                view_descriptor_set_handle,
            );
        }

        mesh_renderer.gen_occlusion_and_cull(
            &render_context,
            &mut render_surface,
            &instance_manager,
        );

        let mut cmd_buffer = render_context.alloc_command_buffer();
        cmd_buffer.bind_index_buffer(&renderer.static_buffer().index_buffer_binding());
        cmd_buffer.bind_vertex_buffers(0, &[instance_manager.vertex_buffer_binding()]);

        let picking_pass = render_surface.picking_renderpass();
        let mut picking_pass = picking_pass.write();
        picking_pass.render(
            &picking_manager,
            &render_context,
            &mut cmd_buffer,
            render_surface.as_mut(),
            &instance_manager,
            q_manipulator_drawables.as_slice(),
            q_lights.as_slice(),
            &mesh_manager,
            &model_manager,
            camera_component,
            &mesh_renderer,
        );

        TmpRenderPass::render(
            &render_context,
            &mut cmd_buffer,
            render_surface.as_mut(),
            &mesh_renderer,
        );

        let debug_renderpass = render_surface.debug_renderpass();
        let debug_renderpass = debug_renderpass.write();
        debug_renderpass.render(
            &render_context,
            &mut cmd_buffer,
            render_surface.as_mut(),
            q_picked_drawables.as_slice(),
            q_manipulator_drawables.as_slice(),
            camera_component,
            &mesh_manager,
            &model_manager,
            &debug_display,
        );

        let egui_pass = render_surface.egui_renderpass();
        let mut egui_pass = egui_pass.write();
        egui_pass.update_font_texture(&render_context, &cmd_buffer, &egui.ctx);
        if egui.enable {
            egui_pass.render(
                &render_context,
                &mut cmd_buffer,
                render_surface.as_mut(),
                &egui,
            );
        }

        // queue
        let sem = render_surface.acquire();
        {
            let graphics_queue = render_context.graphics_queue();
            graphics_queue.submit(&mut [cmd_buffer.finalize()], &[], &[sem], None);

            render_surface.present(&render_context);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn render_end(
    mut renderer: ResMut<'_, Renderer>,
    mut debug_display: ResMut<'_, DebugDisplay>,
    mut descriptor_heap_manager: ResMut<'_, DescriptorHeapManager>,
) {
    descriptor_heap_manager.end_frame();
    debug_display.end_frame();
    renderer.end_frame();
}
