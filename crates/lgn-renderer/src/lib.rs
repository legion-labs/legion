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
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

#[allow(unused_imports)]
use cgen::*;

mod labels;
use components::{MaterialComponent, ModelComponent};
use gpu_renderer::{GpuInstanceManager, MeshRenderer, RenderElement};
pub use labels::*;

mod renderer;
use lgn_data_runtime::ResourceTypeAndId;
use lgn_graphics_api::{AddressMode, CompareOp, FilterType, MipMapMode, ResourceUsage, SamplerDef};
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_math::{Vec2, Vec4};
pub use renderer::*;

mod render_context;
pub use render_context::*;

pub mod resources;
use resources::{
    BindlessTextureManager, DescriptorHeapManager, GpuDataPlugin, GpuEntityColorManager,
    GpuEntityTransformManager, GpuMaterialManager, GpuPickingDataManager, ModelManager,
    PersistentDescriptorSetManager, PipelineManager,
};

pub mod components;

pub mod gpu_renderer;

pub mod picking;

pub mod debug_display;
pub mod egui;

pub mod hl_gfx_api;

pub(crate) mod lighting;
pub(crate) mod render_pass;

use crate::{
    components::{
        debug_display_lights, ui_lights, update_lights, ManipulatorComponent, PickedComponent,
        RenderSurfaceCreatedForWindow, RenderSurfaceExtents, RenderSurfaces,
    },
    egui::egui_plugin::{Egui, EguiPlugin},
    gpu_renderer::{GpuInstanceVas, MeshRendererPlugin},
    lighting::LightingManager,
    picking::{ManipulatorManager, PickingIdContext, PickingManager, PickingPlugin},
    render_pass::TmpRenderPass,
    resources::{IndexBlock, MeshManager},
    RenderStage,
};
use lgn_app::{App, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_math::{const_vec3, Vec3};
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;
use lgn_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};

use crate::debug_display::DebugDisplay;
use crate::resources::{Mesh, ModelMetaData, UniformGPUDataUpdater};

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
pub struct RendererPlugin {
    // tbd: move in RendererOptions
    _egui_enabled: bool,
    // tbd: remove
    runs_dynamic_systems: bool,
}

impl RendererPlugin {
    pub fn new(egui_enabled: bool, runs_dynamic_systems: bool) -> Self {
        Self {
            _egui_enabled: egui_enabled,
            runs_dynamic_systems,
        }
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        const NUM_RENDER_FRAMES: usize = 2;
        let renderer = Renderer::new(NUM_RENDER_FRAMES);
        let device_context = renderer.device_context().clone();
        let static_buffer = renderer.static_buffer().clone();
        let descriptor_heap_manager =
            DescriptorHeapManager::new(NUM_RENDER_FRAMES, &device_context);
        let pipeline_manager = PipelineManager::new(&device_context);
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
        // Resources
        //
        app.insert_resource(MeshRenderer::new(&static_buffer));
        app.insert_resource(pipeline_manager);
        app.insert_resource(ManipulatorManager::new());
        app.insert_resource(CGenRegistryList::new());
        app.insert_resource(RenderSurfaces::new());
        app.insert_resource(ModelManager::new());
        app.insert_resource(MeshManager::new(&renderer));
        app.insert_resource(BindlessTextureManager::new(renderer.device_context(), 256));
        app.insert_resource(DebugDisplay::default());
        app.insert_resource(LightingManager::default());
        app.insert_resource(GpuInstanceManager::new(&static_buffer));
        app.insert_resource(MissingVisualTracker::default());
        app.insert_resource(descriptor_heap_manager);
        app.insert_resource(PersistentDescriptorSetManager::new());
        app.add_plugin(EguiPlugin::new());
        app.add_plugin(PickingPlugin {});
        app.add_plugin(GpuDataPlugin::new(&static_buffer));
        app.add_plugin(MeshRendererPlugin {});
        app.insert_resource(renderer);

        //
        // Events
        //
        app.add_event::<RenderSurfaceCreatedForWindow>();

        //
        // Stage Startup
        //
        app.add_startup_system(init_cgen);
        app.add_startup_system(init_manipulation_manager);
        app.add_startup_system(create_camera);

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
        if self.runs_dynamic_systems {
            app.add_system_to_stage(RenderStage::Prepare, ui_lights);
        }
        app.add_system_to_stage(RenderStage::Prepare, debug_display_lights);
        app.add_system_to_stage(RenderStage::Prepare, update_models);
        app.add_system_to_stage(RenderStage::Prepare, update_gpu_instances);
        app.add_system_to_stage(RenderStage::Prepare, update_missing_visuals);
        app.add_system_to_stage(RenderStage::Prepare, update_lights);
        app.add_system_to_stage(RenderStage::Prepare, camera_control);
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
fn init_cgen(
    renderer: Res<'_, Renderer>,
    mut pipeline_manager: ResMut<'_, PipelineManager>,
    mut cgen_registries: ResMut<'_, CGenRegistryList>,
    descriptor_heap_manager: Res<'_, DescriptorHeapManager>,
    mut persistent_descriptor_set_manager: ResMut<'_, PersistentDescriptorSetManager>,
) {
    let cgen_registry = Arc::new(cgen::initialize(renderer.device_context()));
    pipeline_manager.register_shader_families(&cgen_registry);
    cgen_registries.push(cgen_registry);
    persistent_descriptor_set_manager.initialize(&descriptor_heap_manager);
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

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn update_models(
    renderer: Res<'_, Renderer>,
    mut model_manager: ResMut<'_, ModelManager>,
    mut mesh_manager: ResMut<'_, MeshManager>,
    mut updated_models: Query<'_, '_, &mut ModelComponent, Changed<ModelComponent>>,
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
) {
    for updated_model in updated_models.iter_mut() {
        if let Some(mesh_reference) = &updated_model.model_id {
            missing_visuals_tracker.add_visuals(*mesh_reference);
            let ids = mesh_manager.add_meshes(&renderer, &updated_model.meshes);

            let mut meshes = Vec::new();
            for (idx, _meshes) in updated_model.meshes.iter().enumerate() {
                meshes.push(Mesh {
                    mesh_id: ids[idx],
                    material_id: u32::MAX, //TODO
                });
            }
            model_manager.add_model(*mesh_reference, ModelMetaData { meshes });
        }
    }
}

#[derive(Default)]
struct MissingVisualTracker {
    entities: BTreeMap<ResourceTypeAndId, HashSet<Entity>>,
    visuals_added: Vec<ResourceTypeAndId>,
}

impl MissingVisualTracker {
    fn add_entity(&mut self, resource_id: ResourceTypeAndId, entity_id: Entity) {
        if let Some(entry) = self.entities.get_mut(&resource_id) {
            entry.insert(entity_id);
        } else {
            let mut set = HashSet::new();
            set.insert(entity_id);
            self.entities.insert(resource_id, set);
        }
    }

    fn add_visuals(&mut self, resource_id: ResourceTypeAndId) {
        self.visuals_added.push(resource_id);
    }

    fn get_entities_to_update(&mut self) -> HashSet<Entity> {
        let mut entities = HashSet::new();
        for visual in &self.visuals_added {
            if let Some(entry) = self.entities.get(visual) {
                for entity in entry {
                    entities.insert(*entity);
                }
                self.entities.remove_entry(visual);
            }
        }
        self.visuals_added.clear();
        entities
    }
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn update_missing_visuals(
    mut missing_visuals_tracker: ResMut<'_, MissingVisualTracker>,
    mut visuals_query: Query<
        '_,
        '_,
        (Entity, &mut VisualComponent, Option<&MaterialComponent>),
        Without<ManipulatorComponent>,
    >,
) {
    for entity in missing_visuals_tracker.get_entities_to_update() {
        if let Ok((_entity, mut visual_component, _mat_component)) = visuals_query.get_mut(entity) {
            visual_component.as_mut(); // Will trigger 'changed' to the visual component and it will be updated on the next update_gpu_instances()
        }
    }
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::type_complexity,
    clippy::too_many_arguments
)]
fn update_gpu_instances(
    renderer: Res<'_, Renderer>,
    mut mesh_renderer: ResMut<'_, MeshRenderer>,
    picking_manager: Res<'_, PickingManager>,
    mut picking_data_manager: ResMut<'_, GpuPickingDataManager>,
    mut instance_manager: ResMut<'_, GpuInstanceManager>,
    model_manager: Res<'_, ModelManager>,
    mesh_manager: Res<'_, MeshManager>,
    material_manager: Res<'_, GpuMaterialManager>,
    color_manager: Res<'_, GpuEntityColorManager>,
    transform_manager: Res<'_, GpuEntityTransformManager>,
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

    for (entity, _mesh, mat_component) in instance_query.iter() {
        picking_data_manager.remove_gpu_data(&entity);
        if let Some(removed_ids) = instance_manager.remove_gpu_instance(entity) {
            for removed_id in removed_ids {
                let mut material_key = None;
                if let Some(material) = mat_component {
                    material_key = Some(material.material_id);
                }

                let material_idx = material_manager.id_for_index(material_key, 0);
                mesh_renderer.unregister_element(material_idx, removed_id);
            }
        }
    }

    let mut picking_block: Option<IndexBlock> = None;
    let mut instance_block: Option<IndexBlock> = None;
    for (entity, mesh, mat_component) in instance_query.iter() {
        let color: (f32, f32, f32, f32) = (
            f32::from(mesh.color.r) / 255.0f32,
            f32::from(mesh.color.g) / 255.0f32,
            f32::from(mesh.color.b) / 255.0f32,
            f32::from(mesh.color.a) / 255.0f32,
        );
        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();
        instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());
        instance_color.set_color_blend(if mat_component.is_none() { 1.0 } else { 0.0 }.into());

        color_manager.update_gpu_data(&entity, 0, &[instance_color], &mut updater);

        let mut material_key = None;
        if let Some(material) = mat_component {
            material_key = Some(material.material_id);
        }

        picking_data_manager.alloc_gpu_data(entity, &mut picking_block);

        let mut picking_data = cgen::cgen_type::GpuInstancePickingData::default();
        picking_data.set_picking_id(picking_context.aquire_picking_id(entity).into());
        picking_data_manager.update_gpu_data(&entity, 0, &[picking_data], &mut updater);

        let (model_meta_data, ready) = model_manager.get_model_meta_data(mesh);
        if !ready {
            if let Some(reference) = &mesh.model_reference {
                missing_visuals_tracker.add_entity(*reference, entity);
            }
        }
        for mesh in &model_meta_data.meshes {
            let mesh_meta_data = mesh_manager.get_mesh_meta_data(mesh.mesh_id);
            let instance_vas = GpuInstanceVas {
                submesh_va: mesh_meta_data.mesh_description_offset,
                material_va: material_manager.va_for_index(material_key, 0) as u32,
                color_va: color_manager.va_for_index(Some(entity), 0) as u32,
                transform_va: transform_manager.va_for_index(Some(entity), 0) as u32,
                picking_data_va: picking_data_manager.va_for_index(Some(entity), 0) as u32,
            };

            let gpu_instance_id = instance_manager.add_gpu_instance(
                entity,
                &mut instance_block,
                &mut updater,
                &instance_vas,
            );

            let material_idx = material_manager.id_for_index(material_key, 0);
            mesh_renderer.register_material(material_idx);
            mesh_renderer.register_element(
                material_idx,
                &RenderElement::new(gpu_instance_id, mesh.mesh_id as u32, &mesh_manager),
            );
        }
    }
    instance_manager.return_index_block(instance_block);
    picking_data_manager.return_index_block(picking_block);

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
        Res<'_, BindlessTextureManager>,
        Res<'_, PipelineManager>,
        Res<'_, MeshRenderer>,
        Res<'_, MeshManager>,
        Res<'_, PickingManager>,
        Res<'_, GpuInstanceManager>,
        Res<'_, Egui>,
        Res<'_, DebugDisplay>,
        Res<'_, LightingManager>,
        Res<'_, DescriptorHeapManager>,
        ResMut<'_, ModelManager>,
    ),
    queries: (
        Query<'_, '_, &mut RenderSurface>,
        Query<'_, '_, (Entity, &VisualComponent), Without<ManipulatorComponent>>,
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
    let bindless_textures = resources.1;
    let pipeline_manager = resources.2;
    let mesh_renderer = resources.3;
    let mesh_manager = resources.4;
    let picking_manager = resources.5;
    let instance_manager = resources.6;
    let egui = resources.7;
    let debug_display = resources.8;
    let lighting_manager = resources.9;
    let descriptor_heap_manager = resources.10;
    let model_manager = resources.11;

    // queries
    let mut q_render_surfaces = queries.0;
    let q_drawables = queries.1;
    let q_picked_drawables = queries.2;
    let q_manipulator_drawables = queries.3;
    let q_lights = queries.4;
    let q_cameras = queries.5;

    // start
    let mut render_context =
        RenderContext::new(&renderer, &descriptor_heap_manager, &pipeline_manager);
    let q_drawables = q_drawables
        .iter()
        .collect::<Vec<(Entity, &VisualComponent)>>();
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

        let default_black_texture = bindless_textures.default_black_texture_view();
        let bindlesss_descriptors = bindless_textures.bindless_texures_for_update();

        let mut desc_refs = [&default_black_texture; 256];
        for index in 0..bindlesss_descriptors.len() {
            desc_refs[index] = &bindlesss_descriptors[index];
        }
        frame_descriptor_set.set_material_textures(&desc_refs);

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

            let view_descriptor_set_handle = render_context.write_descriptor_set(
                cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                view_descriptor_set.descriptor_refs(),
            );

            render_context.set_view_descriptor_set(
                cgen::descriptor_set::ViewDescriptorSet::descriptor_set_layout(),
                view_descriptor_set_handle,
            );
        }

        let mut cmd_buffer = render_context.alloc_command_buffer();
        cmd_buffer.bind_vertex_buffers(0, &[instance_manager.vertex_buffer_binding()]);

        let picking_pass = render_surface.picking_renderpass();
        let mut picking_pass = picking_pass.write();
        picking_pass.render(
            &picking_manager,
            &render_context,
            render_surface.as_mut(),
            &instance_manager,
            q_drawables.as_slice(),
            q_manipulator_drawables.as_slice(),
            q_lights.as_slice(),
            &mesh_manager,
            &model_manager,
            camera_component,
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
