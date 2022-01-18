//! Renderer plugin.

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::new_without_default,
    clippy::uninit_vec
)]

#[path = "../codegen/rust/mod.rs"]
mod cgen;
#[allow(unused_imports)]
use cgen::*;

mod labels;
pub use labels::*;

mod renderer;
pub use renderer::*;

mod render_handle;
pub use render_handle::*;

mod render_context;
pub use render_context::*;

pub mod resources;

mod memory;

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
    resources::{DefaultMeshId, DefaultMeshes, MetaCubePlugin},
    RenderStage,
};
use lgn_app::{App, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;
use lgn_transform::components::Transform;
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
        let default_meshes = DefaultMeshes::new(&renderer);

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
        if self.meta_cube_size != 0 {
            app.add_plugin(MetaCubePlugin::new(self.meta_cube_size));
        }

        app.insert_resource(ManipulatorManager::new());
        app.add_startup_system(init_manipulation_manager);

        app.insert_resource(RenderSurfaces::new());
        app.insert_resource(default_meshes);
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
        app.add_system_to_stage(RenderStage::Prepare, update_transform);
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

fn render_pre_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.begin_frame();
}

#[span_fn]
fn update_transform(
    mut renderer: ResMut<'_, Renderer>,
    mut query: Query<
        '_,
        '_,
        (
            Entity,
            &Transform,
            &mut StaticMesh,
            Option<&ManipulatorComponent>,
        ),
        Changed<Transform>,
    >,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 4096 * 1024);
    let mut gpu_data = renderer.acquire_transform_data();

    for (entity, transform, mut mesh, manipulator) in query.iter_mut() {
        if manipulator.is_none() {
            mesh.world_offset = gpu_data.ensure_index_allocated(entity.id()) as u32;

            let mut world = cgen::cgen_type::EntityTransforms::default();
            world.set_world(transform.compute_matrix().into());
            updater.add_update_jobs(&[world], u64::from(mesh.world_offset));
        } else {
            mesh.world_offset = u32::MAX;
        }
    }

    renderer.test_add_update_jobs(updater.job_blocks());
    renderer.release_transform_data(gpu_data);
}

#[span_fn]
#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
fn render_update(
    renderer: ResMut<'_, Renderer>,
    default_meshes: ResMut<'_, DefaultMeshes>,
    picking_manager: ResMut<'_, PickingManager>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    q_drawables: Query<
        '_,
        '_,
        (&StaticMesh, Option<&PickedComponent>),
        Without<ManipulatorComponent>,
    >,
    q_debug_drawables: Query<
        '_,
        '_,
        (&StaticMesh, &Transform, Option<&PickedComponent>),
        Without<ManipulatorComponent>,
    >,
    q_manipulator_drawables: Query<'_, '_, (&StaticMesh, &Transform, &ManipulatorComponent)>,
    lighting_manager: Res<'_, LightingManager>,
    q_lights: Query<'_, '_, (&LightComponent, &Transform)>,
    mut egui: ResMut<'_, Egui>,
    mut debug_display: ResMut<'_, DebugDisplay>,
    q_cameras: Query<'_, '_, &CameraComponent>,
) {
    crate::egui::egui_plugin::end_frame(&mut egui);

    let render_context = RenderContext::new(&renderer);
    let q_drawables = q_drawables
        .iter()
        .collect::<Vec<(&StaticMesh, Option<&PickedComponent>)>>();
    let q_debug_drawables =
        q_debug_drawables
            .iter()
            .collect::<Vec<(&StaticMesh, &Transform, Option<&PickedComponent>)>>();
    let q_manipulator_drawables =
        q_manipulator_drawables
            .iter()
            .collect::<Vec<(&StaticMesh, &Transform, &ManipulatorComponent)>>();
    let q_lights = q_lights
        .iter()
        .collect::<Vec<(&LightComponent, &Transform)>>();

    let q_cameras = q_cameras.iter().collect::<Vec<&CameraComponent>>();
    let default_camera = CameraComponent::default();
    let camera_component = if !q_cameras.is_empty() {
        q_cameras[0]
    } else {
        &default_camera
    };

    let mut light_picking_mesh = StaticMesh::from_default_meshes(
        default_meshes.as_ref(),
        DefaultMeshId::Sphere as usize,
        Color::default(),
    );
    light_picking_mesh.world_offset = 0xffffffff; // will force the shader to use custom made world matrix

    renderer.flush_update_jobs(&render_context);

    // For each surface/view, we have to execute the render graph
    for mut render_surface in q_render_surfaces.iter_mut() {
        // View descriptor set
        /* WIP
        {
            let (view_matrix, projection_matrix) = camera_component.build_view_projection(
                render_surface.extents().width() as f32,
                render_surface.extents().height() as f32,
            );

            let transient_allocator = render_context.transient_buffer_allocator();

            let mut view_data = crate::cgen::cgen_type::ViewData::default();
            view_data.view = view_matrix.into();
            view_data.projection = projection_matrix.into();

            let sub_allocation =
                transient_allocator.copy_data(&view_data, ResourceUsage::AS_CONST_BUFFER);

            let const_buffer_view = sub_allocation.const_buffer_view();

            let mut view_descriptor_set = cgen::descriptor_set::ViewDescriptorSet::default();
            view_descriptor_set.set_view_data(&const_buffer_view);

            let handle = render_context.write_descriptor_set(&view_descriptor_set);
        }
        */

        let mut cmd_buffer = render_context.alloc_command_buffer();
        let picking_pass = render_surface.picking_renderpass();
        let mut picking_pass = picking_pass.write();
        picking_pass.render(
            &picking_manager,
            &render_context,
            render_surface.as_mut(),
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
            camera_component,
            lighting_manager.as_ref(),
        );

        let debug_renderpass = render_surface.debug_renderpass();
        let debug_renderpass = debug_renderpass.write();
        debug_renderpass.render(
            &render_context,
            &mut cmd_buffer,
            render_surface.as_mut(),
            q_debug_drawables.as_slice(),
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
        let graphics_queue = render_context.graphics_queue();
        graphics_queue.submit(&mut [cmd_buffer.finalize()], &[], &[sem], None);

        render_surface.present(&render_context);
    }
}

fn render_post_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.end_frame();
}
