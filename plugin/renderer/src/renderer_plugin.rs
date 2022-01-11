#![allow(unsafe_code)]
use crate::{
    components::{
        DirectionalLight, ManipulatorComponent, OmnidirectionalLight, PickedComponent,
        RenderSurfaceCreatedForWindow, RenderSurfaceExtents, RenderSurfaces, Spotlight,
    },
    egui::egui_plugin::{Egui, EguiPlugin},
    lighting::LightingManager,
    picking::{ManipulatorManager, PickingManager, PickingPlugin},
    resources::DefaultMeshes,
};
use lgn_app::{App, CoreStage, Events, Plugin};

use lgn_ecs::prelude::*;
use lgn_transform::components::Transform;
use lgn_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};

use crate::debug_display::DebugDisplay;
use crate::resources::{EntityTransforms, UniformGPUDataUpdater};

use crate::{
    components::{
        camera_control, create_camera, CameraComponent, LightComponent, LightType, RenderSurface,
        StaticMesh,
    },
    labels::RendererSystemLabel,
    RenderContext, Renderer,
};

#[derive(Default)]
pub struct RendererPlugin {
    enable_egui: bool,
    runs_dynamic_systems: bool,
}

impl RendererPlugin {
    pub fn new(enable_egui: bool, runs_dynamic_systems: bool) -> Self {
        Self {
            enable_egui,
            runs_dynamic_systems,
        }
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        let renderer = Renderer::new().unwrap();
        let default_meshes = DefaultMeshes::new(&renderer);

        app.add_plugin(EguiPlugin::new(self.enable_egui));
        app.add_plugin(PickingPlugin {});

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

        // Update
        if self.runs_dynamic_systems {
            app.add_system(update_lighting_ui.before(RendererSystemLabel::FrameUpdate));
        }
        app.add_system(update_debug.before(RendererSystemLabel::FrameUpdate));
        app.add_system(update_transform.before(RendererSystemLabel::FrameUpdate));
        app.add_system(update_lights.before(RendererSystemLabel::FrameUpdate));
        app.add_system(camera_control.before(RendererSystemLabel::FrameUpdate));
        app.add_system(on_window_created.exclusive_system());
        app.add_system(on_window_resized.exclusive_system());
        app.add_system(on_window_close_requested.exclusive_system());

        app.add_system_set(
            SystemSet::new()
                .with_system(render_update)
                .label(RendererSystemLabel::FrameUpdate),
        );

        // Post-Update
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            render_post_update, // .label(RendererSystemLabel::FrameDone),
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

fn init_manipulation_manager(
    commands: Commands<'_, '_>,
    mut manipulation_manager: ResMut<'_, ManipulatorManager>,
    default_meshes: Res<'_, DefaultMeshes>,
    picking_manager: Res<'_, PickingManager>,
) {
    manipulation_manager.initialize(commands, default_meshes, picking_manager);
}

#[allow(clippy::needless_pass_by_value)]
fn update_lighting_ui(
    egui_ctx: Res<'_, Egui>,
    mut lights: Query<'_, '_, (&mut LightComponent, &mut Transform)>,
    mut lighting_manager: ResMut<'_, LightingManager>,
) {
    egui::Window::new("Lights").show(&egui_ctx.ctx, |ui| {
        ui.checkbox(&mut lighting_manager.diffuse, "Diffuse");
        ui.checkbox(&mut lighting_manager.specular, "Specular");
        ui.add(
            egui::Slider::new(&mut lighting_manager.specular_reflection, 0.0..=1.0)
                .text("specular_reflection"),
        );
        ui.add(
            egui::Slider::new(&mut lighting_manager.diffuse_reflection, 0.0..=1.0)
                .text("diffuse_reflection"),
        );
        ui.add(
            egui::Slider::new(&mut lighting_manager.ambient_reflection, 0.0..=1.0)
                .text("ambient_reflection"),
        );
        ui.add(egui::Slider::new(&mut lighting_manager.shininess, 1.0..=32.0).text("shininess"));
        ui.label("Lights");
        for (i, (mut light, mut transform)) in lights.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::Checkbox::new(&mut light.enabled, "Enabled"));
                match light.light_type {
                    LightType::Directional { ref mut direction } => {
                        ui.label(format!("Light {} (dir): ", i));
                        ui.add(egui::Slider::new(&mut direction.x, -1.0..=1.0).text("x"));
                        ui.add(egui::Slider::new(&mut direction.y, -1.0..=1.0).text("y"));
                        ui.add(egui::Slider::new(&mut direction.z, -1.0..=1.0).text("z"));
                    }
                    LightType::Omnidirectional { .. } => {
                        ui.label(format!("Light {} (omni): ", i));
                        ui.add(
                            egui::Slider::new(&mut transform.translation.x, -10.0..=10.0).text("x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut transform.translation.y, -10.0..=10.0).text("y"),
                        );
                        ui.add(
                            egui::Slider::new(&mut transform.translation.z, -10.0..=10.0).text("z"),
                        );
                        ui.add(
                            egui::Slider::new(&mut light.radiance, 0.0..=300.0).text("radiance"),
                        );
                    }
                    LightType::Spotlight {
                        ref mut direction,
                        ref mut cone_angle,
                        ..
                    } => {
                        ui.label(format!("Light {} (spot): ", i));
                        ui.add(egui::Slider::new(&mut direction.x, -1.0..=1.0).text("x"));
                        ui.add(egui::Slider::new(&mut direction.y, -1.0..=1.0).text("y"));
                        ui.add(egui::Slider::new(&mut direction.z, -1.0..=1.0).text("z"));
                        ui.add(
                            egui::Slider::new(&mut transform.translation.x, -10.0..=10.0).text("x"),
                        );
                        ui.add(
                            egui::Slider::new(&mut transform.translation.y, -10.0..=10.0).text("y"),
                        );
                        ui.add(
                            egui::Slider::new(&mut transform.translation.z, -10.0..=10.0).text("z"),
                        );
                        ui.add(
                            egui::Slider::new(cone_angle, -0.0..=std::f32::consts::PI)
                                .text("angle"),
                        );
                    }
                }
                let mut rgb = [light.color.0, light.color.1, light.color.2];
                if ui.color_edit_button_rgb(&mut rgb).changed() {
                    light.color.0 = rgb[0];
                    light.color.1 = rgb[1];
                    light.color.2 = rgb[2];
                }
            });
        }
    });
}

#[allow(clippy::match_same_arms)] // TODO: remove when more advanced visualization is introduced
#[allow(clippy::needless_pass_by_value)]
fn update_debug(
    renderer: Res<'_, Renderer>,
    mut debug_display: ResMut<'_, DebugDisplay>,
    lights: Query<'_, '_, (&LightComponent, &Transform)>,
) {
    let bump = renderer.acquire_bump_allocator();
    debug_display.create_display_list(bump.bump(), |display_list| {
        for (light, transform) in lights.iter() {
            display_list.add_cube(transform.translation, bump.bump());
            match light.light_type {
                LightType::Directional { direction } => display_list.add_arrow(
                    transform.translation,
                    transform.translation - direction.normalize(),
                    bump.bump(),
                ),
                LightType::Spotlight { direction, .. } => display_list.add_arrow(
                    transform.translation,
                    transform.translation - direction.normalize(),
                    bump.bump(),
                ),
                LightType::Omnidirectional { .. } => (),
            }
        }
    });
    renderer.release_bump_allocator(bump);
}

fn render_pre_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.begin_frame();
}

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
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let mut gpu_data = renderer.acquire_transform_data();

    for (entity, transform, mut mesh, manipulator) in query.iter_mut() {
        if manipulator.is_none() {
            mesh.world_offset = gpu_data.ensure_index_allocated(entity.id()) as u32;

            let world = EntityTransforms {
                world: transform.compute_matrix(),
            };

            updater.add_update_jobs(&[world], u64::from(mesh.world_offset));
        } else {
            mesh.world_offset = u32::MAX;
        }
    }

    renderer.test_add_update_jobs(updater.job_blocks());
    renderer.release_transform_data(gpu_data);
}

#[allow(clippy::needless_pass_by_value)]
fn update_lights(
    mut renderer: ResMut<'_, Renderer>,
    query: Query<'_, '_, (Entity, &Transform, &LightComponent)>,
    mut lighting_manager: ResMut<'_, LightingManager>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let omnidirectional_gpu_data = renderer.acquire_omnidirectional_lights_data();
    let directional_gpu_data = renderer.acquire_directional_lights_data();
    let spotlights_gpu_data = renderer.acquire_spotlights_data();

    const NUM_LIGHTS: usize = 8;

    let mut omnidirectional_lights_data =
        Vec::<f32>::with_capacity(OmnidirectionalLight::SIZE * NUM_LIGHTS);
    let mut directional_lights_data =
        Vec::<f32>::with_capacity(DirectionalLight::SIZE * NUM_LIGHTS);
    let mut spotlights_data = Vec::<f32>::with_capacity(Spotlight::SIZE * NUM_LIGHTS);
    let mut num_directional_lights = 0;
    let mut num_omnidirectional_lights = 0;
    let mut num_spotlights = 0;

    for (_entity, transform, light) in query.iter() {
        if !light.enabled {
            continue;
        }
        match light.light_type {
            LightType::Directional { direction } => {
                directional_lights_data.push(direction.x);
                directional_lights_data.push(direction.y);
                directional_lights_data.push(direction.z);
                directional_lights_data.push(light.radiance);
                directional_lights_data.push(light.color.0);
                directional_lights_data.push(light.color.1);
                directional_lights_data.push(light.color.2);
                num_directional_lights += 1;
                unsafe {
                    directional_lights_data
                        .set_len(DirectionalLight::SIZE / 4 * num_directional_lights as usize);
                }
            }
            LightType::Omnidirectional => {
                omnidirectional_lights_data.push(transform.translation.x);
                omnidirectional_lights_data.push(transform.translation.y);
                omnidirectional_lights_data.push(transform.translation.z);
                omnidirectional_lights_data.push(light.radiance);
                omnidirectional_lights_data.push(light.color.0);
                omnidirectional_lights_data.push(light.color.1);
                omnidirectional_lights_data.push(light.color.2);
                num_omnidirectional_lights += 1;
                unsafe {
                    omnidirectional_lights_data.set_len(
                        OmnidirectionalLight::SIZE / 4 * num_omnidirectional_lights as usize,
                    );
                }
            }
            LightType::Spotlight {
                direction,
                cone_angle,
            } => {
                spotlights_data.push(transform.translation.x);
                spotlights_data.push(transform.translation.y);
                spotlights_data.push(transform.translation.z);
                spotlights_data.push(light.radiance);
                spotlights_data.push(direction.x);
                spotlights_data.push(direction.y);
                spotlights_data.push(direction.z);
                spotlights_data.push(cone_angle);
                spotlights_data.push(light.color.0);
                spotlights_data.push(light.color.1);
                spotlights_data.push(light.color.2);
                num_spotlights += 1;
                unsafe {
                    spotlights_data.set_len(Spotlight::SIZE / 4 * num_spotlights as usize);
                }
            }
        }
    }

    lighting_manager.num_directional_lights = num_directional_lights;
    lighting_manager.num_omnidirectional_lights = num_omnidirectional_lights;
    lighting_manager.num_spotlights = num_spotlights;

    if !omnidirectional_lights_data.is_empty() {
        updater.add_update_jobs(
            &omnidirectional_lights_data,
            omnidirectional_gpu_data.offset(),
        );
    }
    if !directional_lights_data.is_empty() {
        updater.add_update_jobs(&directional_lights_data, directional_gpu_data.offset());
    }
    if !spotlights_data.is_empty() {
        updater.add_update_jobs(&spotlights_data, spotlights_gpu_data.offset());
    }
    renderer.test_add_update_jobs(updater.job_blocks());

    renderer.release_omnidirectional_lights_data(omnidirectional_gpu_data);
    renderer.release_directional_lights_data(directional_gpu_data);
    renderer.release_spotlights_data(spotlights_gpu_data);
}

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
    task_pool: Res<'_, crate::RenderTaskPool>,
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

    let q_cameras = q_cameras.iter().collect::<Vec<&CameraComponent>>();
    let default_camera = CameraComponent::default();
    let camera_component = if !q_cameras.is_empty() {
        q_cameras[0]
    } else {
        &default_camera
    };

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

        let cmd_buffer = render_context.alloc_command_buffer();
        let picking_pass = render_surface.picking_renderpass();
        let mut picking_pass = picking_pass.write();
        picking_pass.render(
            &picking_manager,
            &render_context,
            render_surface.as_mut(),
            q_drawables.as_slice(),
            q_manipulator_drawables.as_slice(),
            camera_component,
        );

        let render_pass = render_surface.test_renderpass();
        let render_pass = render_pass.write();
        render_pass.render(
            &render_context,
            &cmd_buffer,
            render_surface.as_mut(),
            q_drawables.as_slice(),
            camera_component,
            lighting_manager.as_ref(),
        );

        let debug_renderpass = render_surface.debug_renderpass();
        let debug_renderpass = debug_renderpass.write();
        debug_renderpass.render(
            &render_context,
            &cmd_buffer,
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
            egui_pass.render(&render_context, &cmd_buffer, render_surface.as_mut(), &egui);
        }

        // queue
        let sem = render_surface.acquire();
        let graphics_queue = render_context.graphics_queue();
        graphics_queue.submit(&mut [cmd_buffer.finalize()], &[], &[sem], None);

        render_surface.present(&render_context, &task_pool);
    }
}

fn render_post_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.end_frame();
}
