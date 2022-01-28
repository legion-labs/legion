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
#[allow(unused_imports)]
use cgen::*;

mod labels;
use components::MaterialComponent;
pub use labels::*;

mod renderer;
use lgn_core::BumpAllocatorPool;
use lgn_graphics_api::ResourceUsage;
use lgn_math::{Vec2, Vec4};
pub use renderer::*;

mod render_context;
pub use render_context::*;
use resources::{
    DefaultMaterials, GpuDataPlugin, GpuInstanceColor, GpuInstanceIdAllocator,
    GpuInstancePickingData, GpuInstanceTransform, GpuInstanceVATable, GpuVaTableForGpuInstance,
    MaterialManager,
};

pub mod resources;

pub mod components;

pub mod picking;

pub mod static_mesh_render_data;

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
    lighting::LightingManager,
    picking::{ManipulatorManager, PickingManager, PickingPlugin},
    resources::{DefaultMaterialType, DefaultMeshType, DefaultMeshes, MetaCubePlugin},
    RenderStage,
};
use lgn_app::{App, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;
use lgn_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};

use crate::debug_display::DebugDisplay;
use crate::resources::UniformGPUDataUpdater;

use crate::{
    components::{
        camera_control, create_camera, CameraComponent, LightComponent, RenderSurface, StaticMesh,
    },
    labels::CommandBufferLabel,
};

#[derive(Default)]
pub struct RendererPlugin {
    enable_egui: bool,
    runs_dynamic_systems: bool,
    meta_cube_size: usize,
}

impl RendererPlugin {
    pub fn new(enable_egui: bool, runs_dynamic_systems: bool, meta_cube_size: usize) -> Self {
        Self {
            enable_egui,
            runs_dynamic_systems,
            meta_cube_size,
        }
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        let renderer = Renderer::new().unwrap();

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

        app.add_plugin(EguiPlugin::new(self.enable_egui));
        app.add_plugin(PickingPlugin {});
        app.add_plugin(GpuDataPlugin::new(renderer.static_buffer()));

        if self.meta_cube_size != 0 {
            app.add_plugin(MetaCubePlugin::new(self.meta_cube_size));
        }

        app.insert_resource(ManipulatorManager::new());
        app.add_startup_system(init_manipulation_manager);
        app.add_startup_system(init_default_materials);

        app.insert_resource(RenderSurfaces::new());
        app.insert_resource(DefaultMeshes::new(&renderer));
        app.insert_resource(DefaultMaterials::new());
        app.insert_resource(MaterialManager::new(renderer.static_buffer()));
        app.insert_resource(renderer);

        app.init_resource::<DebugDisplay>();
        app.init_resource::<LightingManager>();
        app.add_startup_system(create_camera);

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update);
        app.add_system_to_stage(CoreStage::PostUpdate, on_window_created.exclusive_system());
        app.add_system_to_stage(CoreStage::PostUpdate, on_window_resized.exclusive_system());
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            on_window_close_requested.exclusive_system(),
        );

        // Update
        if self.runs_dynamic_systems {
            app.add_system_to_stage(RenderStage::Prepare, ui_lights);
        }
        app.add_system_to_stage(RenderStage::Prepare, debug_display_lights);

        app.add_system_to_stage(
            RenderStage::Prepare,
            add_gpu_instances.label(PrepareLabel::AddedStaticMeshes),
        );
        app.add_system_to_stage(
            RenderStage::Prepare,
            update_transform.after(PrepareLabel::AddedStaticMeshes),
        );
        app.add_system_to_stage(
            RenderStage::Prepare,
            update_materials.before(PrepareLabel::UpdateInstanceIds),
        );
        app.add_system_to_stage(
            RenderStage::Prepare,
            update_gpu_instance_ids
                .label(PrepareLabel::UpdateInstanceIds)
                .after(PrepareLabel::AddedStaticMeshes),
        );

        app.add_system_to_stage(RenderStage::Prepare, update_lights);
        app.add_system_to_stage(RenderStage::Prepare, camera_control);

        app.add_system_set_to_stage(
            RenderStage::Render,
            SystemSet::new()
                .with_system(render_update)
                .before(CommandBufferLabel::Submit)
                .label(CommandBufferLabel::Generate),
        );

        // Post-Update
        app.add_system_to_stage(
            RenderStage::Render,
            render_post_update.label(CommandBufferLabel::Submit),
        );

        app.add_event::<RenderSurfaceCreatedForWindow>();
    }
}

#[allow(clippy::needless_pass_by_value)]
fn on_window_created(
    mut commands: Commands<'_, '_>,
    mut event_window_created: EventReader<'_, '_, WindowCreated>,
    window_list: Res<'_, Windows>,
    renderer: Res<'_, Renderer>,
    mut render_surfaces: ResMut<'_, RenderSurfaces>,
    mut event_render_surface_created: ResMut<'_, Events<RenderSurfaceCreatedForWindow>>,
) {
    for ev in event_window_created.iter() {
        let wnd = window_list.get(ev.id).unwrap();
        let extents = RenderSurfaceExtents::new(wnd.physical_width(), wnd.physical_height());
        let render_surface = RenderSurface::new(&renderer, extents);

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

    drop(query_render_surface);
}

fn init_manipulation_manager(
    commands: Commands<'_, '_>,
    mut manipulation_manager: ResMut<'_, ManipulatorManager>,
    default_meshes: Res<'_, DefaultMeshes>,
    picking_manager: Res<'_, PickingManager>,
) {
    manipulation_manager.initialize(commands, default_meshes, picking_manager);
}

fn init_default_materials(
    commands: Commands<'_, '_>,
    mut default_materials: ResMut<'_, DefaultMaterials>,
) {
    default_materials.initialize(commands);
}

#[allow(clippy::needless_pass_by_value)]
fn render_pre_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.begin_frame();
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn update_transform(
    renderer: Res<'_, Renderer>,
    query: Query<
        '_,
        '_,
        (&GlobalTransform, &StaticMesh, Option<&ManipulatorComponent>),
        Changed<GlobalTransform>,
    >,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for (transform, mesh, manipulator) in query.iter() {
        if manipulator.is_none() {
            let mut world = cgen::cgen_type::GpuInstanceTransform::default();
            world.set_world(transform.compute_matrix().into());
            updater.add_update_jobs(&[world], u64::from(mesh.world_transform_va));
        }
    }

    renderer.add_update_job_block(updater.job_blocks());
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn update_materials(
    renderer: ResMut<'_, Renderer>,
    material_manager: ResMut<'_, MaterialManager>,
    updated_materials: Query<'_, '_, &mut MaterialComponent, Changed<MaterialComponent>>,
) {
    material_manager.update_gpu_data(&renderer, updated_materials);
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_arguments)]
fn add_gpu_instances(
    renderer: Res<'_, Renderer>,
    picking_manager: Res<'_, PickingManager>,
    instance_id_allocator: Res<'_, GpuInstanceIdAllocator>,
    va_table_adresses: Res<'_, GpuVaTableForGpuInstance>,
    mut instance_transforms: ResMut<'_, GpuInstanceTransform>,
    mut instance_offsets: ResMut<'_, GpuInstanceVATable>,
    mut instance_color: ResMut<'_, GpuInstanceColor>,
    mut picking_data: ResMut<'_, GpuInstancePickingData>,
    mut instance_query: Query<
        '_,
        '_,
        (Entity, &mut StaticMesh, Option<&ManipulatorComponent>),
        Added<StaticMesh>,
    >,
) {
    let mut index_block = instance_id_allocator.acquire_index_block();
    let mut picking_block = picking_manager.acquire_picking_id_block();
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for (entity, mut mesh, manipulator) in instance_query.iter_mut() {
        let (new_index_block, new_instance_id) = instance_id_allocator.acquire_index(index_block);

        index_block = new_index_block;
        mesh.gpu_instance_id = new_instance_id;

        mesh.va_table_address =
            instance_offsets.ensure_index_allocated(mesh.gpu_instance_id) as u32;
        mesh.instance_color_va = instance_color.ensure_index_allocated(mesh.gpu_instance_id) as u32;

        if manipulator.is_none() {
            mesh.world_transform_va =
                instance_transforms.ensure_index_allocated(mesh.gpu_instance_id) as u32;
        } else {
            mesh.world_transform_va = u32::MAX;
        }

        va_table_adresses.set_va_table_address_for_gpu_instance(
            &mut updater,
            mesh.gpu_instance_id,
            mesh.va_table_address,
        );

        mesh.picking_id = u32::MAX;
        while mesh.picking_id == u32::MAX {
            if let Some(picking_id) = picking_block.acquire_picking_id(entity) {
                mesh.picking_id = picking_id;
            } else {
                picking_manager.release_picking_id_block(picking_block);
                picking_block = picking_manager.acquire_picking_id_block();
            }
        }

        mesh.picking_data_va = picking_data.ensure_index_allocated(mesh.gpu_instance_id) as u32;
        updater.add_update_jobs(&[mesh.picking_id], u64::from(mesh.picking_data_va));
    }

    renderer.add_update_job_block(updater.job_blocks());
    picking_manager.release_picking_id_block(picking_block);
    instance_id_allocator.release_index_block(index_block);
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn update_gpu_instance_ids(
    renderer: Res<'_, Renderer>,
    default_materials: ResMut<'_, DefaultMaterials>,

    material_query: Query<'_, '_, &MaterialComponent>,
    instance_query: Query<'_, '_, &StaticMesh, Changed<StaticMesh>>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for mesh in instance_query.iter() {
        let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();

        gpu_instance_va_table.set_vertex_buffer_va(mesh.vertex_buffer_va.into());
        gpu_instance_va_table.set_world_transform_va(mesh.world_transform_va.into());

        if let Ok(material) =
            material_query.get(default_materials.get_material_id(mesh.material_type))
        {
            gpu_instance_va_table.set_material_data_va(material.gpu_offset().into());
        } else {
            let default_material =
                material_query.get(default_materials.get_material_id(DefaultMaterialType::Default));

            gpu_instance_va_table
                .set_material_data_va(default_material.unwrap().gpu_offset().into());
        }

        let mut instance_color = cgen::cgen_type::GpuInstanceColor::default();

        let color: (f32, f32, f32, f32) = (
            f32::from(mesh.color.r) / 255.0f32,
            f32::from(mesh.color.g) / 255.0f32,
            f32::from(mesh.color.b) / 255.0f32,
            f32::from(mesh.color.a) / 255.0f32,
        );

        instance_color.set_color(Vec4::new(color.0, color.1, color.2, color.3).into());
        instance_color.set_color_blend(
            if mesh.material_type == DefaultMaterialType::Default {
                1.0
            } else {
                0.0
            }
            .into(),
        );
        updater.add_update_jobs(&[instance_color], u64::from(mesh.instance_color_va));

        gpu_instance_va_table.set_instance_color_va(mesh.instance_color_va.into());
        gpu_instance_va_table.set_picking_data_va(mesh.picking_data_va.into());

        updater.add_update_jobs(&[gpu_instance_va_table], u64::from(mesh.va_table_address));
    }
    renderer.add_update_job_block(updater.job_blocks());
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
fn render_update(
    renderer: ResMut<'_, Renderer>,
    bump_allocator_pool: ResMut<'_, BumpAllocatorPool>,
    default_meshes: ResMut<'_, DefaultMeshes>,
    picking_manager: ResMut<'_, PickingManager>,
    va_table_adresses: Res<'_, GpuVaTableForGpuInstance>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    q_drawables: Query<'_, '_, &StaticMesh, Without<ManipulatorComponent>>,
    q_picked_drawables: Query<
        '_,
        '_,
        (&StaticMesh, &GlobalTransform),
        (With<PickedComponent>, Without<ManipulatorComponent>),
    >,
    q_manipulator_drawables: Query<'_, '_, (&StaticMesh, &GlobalTransform, &ManipulatorComponent)>,
    lighting_manager: Res<'_, LightingManager>,
    q_lights: Query<'_, '_, (&LightComponent, &GlobalTransform)>,
    mut egui: ResMut<'_, Egui>,
    mut debug_display: ResMut<'_, DebugDisplay>,
    q_cameras: Query<'_, '_, &CameraComponent>,
) {
    crate::egui::egui_plugin::end_frame(&mut egui);

    let mut render_context = RenderContext::new(&renderer, &bump_allocator_pool);
    let q_drawables = q_drawables.iter().collect::<Vec<&StaticMesh>>();
    let q_picked_drawables = q_picked_drawables
        .iter()
        .collect::<Vec<(&StaticMesh, &GlobalTransform)>>();
    let q_manipulator_drawables =
        q_manipulator_drawables
            .iter()
            .collect::<Vec<(&StaticMesh, &GlobalTransform, &ManipulatorComponent)>>();
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

    let mut light_picking_mesh = StaticMesh::from_default_meshes(
        default_meshes.as_ref(),
        DefaultMeshType::Sphere as usize,
        Color::default(),
        DefaultMaterialType::Default,
    );
    light_picking_mesh.world_transform_va = 0xffffffff; // will force the shader to use custom made world matrix

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

        let frame_descriptor_set_handle =
            render_context.write_descriptor_set(&frame_descriptor_set);

        render_context.set_frame_descriptor_set_handle(frame_descriptor_set_handle);
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

            let view_descriptor_set_handle =
                render_context.write_descriptor_set(&view_descriptor_set);

            render_context.set_view_descriptor_set_handle(view_descriptor_set_handle);
        }

        let mut cmd_buffer = render_context.alloc_command_buffer();
        cmd_buffer.bind_vertex_buffers(0, &[va_table_adresses.vertex_buffer_binding()]);

        let picking_pass = render_surface.picking_renderpass();
        let mut picking_pass = picking_pass.write();
        picking_pass.render(
            &picking_manager,
            &render_context,
            render_surface.as_mut(),
            &va_table_adresses,
            q_drawables.as_slice(),
            q_manipulator_drawables.as_slice(),
            q_lights.as_slice(),
            &light_picking_mesh,
            camera_component,
        );

        let render_pass = render_surface.test_renderpass();
        let render_pass = render_pass.write();
        render_pass.render(
            &render_context,
            &mut cmd_buffer,
            render_surface.as_mut(),
            q_drawables.as_slice(),
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
            &default_meshes,
            debug_display.as_mut(),
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
    debug_display.clear();
    render_context.release_bump_allocator(&bump_allocator_pool);
}

#[allow(clippy::needless_pass_by_value)]
fn render_post_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.end_frame();
}
