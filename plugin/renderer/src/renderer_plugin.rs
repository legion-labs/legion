use crate::{
    components::{ManipulatorComponent, PickedComponent},
    egui::egui_plugin::{Egui, EguiPlugin},
    picking::{ManipulatorManager, PickingManager, PickingPlugin},
    resources::DefaultMeshes,
};
use lgn_app::{App, CoreStage, Plugin};
use lgn_ecs::prelude::*;
use lgn_math::{EulerRot, Quat};
use lgn_transform::components::Transform;

use crate::debug_display::DebugDisplay;
use crate::resources::{EntityTransforms, UniformGPUDataUpdater};

use crate::{
    components::{
        camera_control, create_camera, CameraComponent, LightComponent, LightSettings, LightType,
        RenderSurface, RotationComponent, StaticMesh,
    },
    labels::RendererSystemLabel,
    RenderContext, Renderer,
};

#[derive(Default)]
pub struct RendererPlugin {
    has_window: bool,
    enable_egui: bool,
    runs_dynamic_systems: bool,
}

impl RendererPlugin {
    pub fn new(has_window: bool, enable_egui: bool, runs_dynamic_systems: bool) -> Self {
        Self {
            has_window,
            enable_egui,
            runs_dynamic_systems,
        }
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        let renderer = Renderer::new().unwrap();
        let default_meshes = DefaultMeshes::new(&renderer);

        app.add_plugin(EguiPlugin::new(self.has_window, self.enable_egui));
        app.add_plugin(PickingPlugin::new(self.has_window));

        app.insert_resource(ManipulatorManager::new());
        app.add_startup_system(init_manipulation_manager);

        app.insert_resource(default_meshes);
        app.insert_resource(renderer);
        app.init_resource::<DebugDisplay>();
        app.init_resource::<LightSettings>();

        app.add_startup_system(create_camera.system());

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update);

        // Update
        if self.runs_dynamic_systems {
            app.add_system(update_rotation.before(RendererSystemLabel::FrameUpdate));
            app.add_system(update_ui.before(RendererSystemLabel::FrameUpdate));
        }
        app.add_system(update_debug.before(RendererSystemLabel::FrameUpdate));
        app.add_system(update_transform.before(RendererSystemLabel::FrameUpdate));
        app.add_system(camera_control.before(RendererSystemLabel::FrameUpdate));

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
fn update_ui(
    egui_ctx: Res<'_, Egui>,
    mut rotations: Query<'_, '_, &mut RotationComponent>,
    mut lights: Query<'_, '_, (&mut LightComponent, &mut Transform)>,
    mut light_settings: ResMut<'_, LightSettings>,
) {
    egui::Window::new("Scene ").show(&egui_ctx.ctx, |ui| {
        ui.label("Objects");
        for (i, mut rotation_component) in rotations.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Object {}: ", i));
                ui.add(
                    egui::Slider::new(&mut rotation_component.rotation_speed.0, 0.0..=5.0)
                        .text("x"),
                );
                ui.add(
                    egui::Slider::new(&mut rotation_component.rotation_speed.1, 0.0..=5.0)
                        .text("y"),
                );
                ui.add(
                    egui::Slider::new(&mut rotation_component.rotation_speed.2, 0.0..=5.0)
                        .text("z"),
                );
            });
        }

        ui.checkbox(&mut light_settings.diffuse, "Diffuse");
        ui.checkbox(&mut light_settings.specular, "Specular");
        ui.add(
            egui::Slider::new(&mut light_settings.specular_reflection, 0.0..=1.0)
                .text("specular_reflection"),
        );
        ui.add(
            egui::Slider::new(&mut light_settings.diffuse_reflection, 0.0..=1.0)
                .text("diffuse_reflection"),
        );
        ui.add(
            egui::Slider::new(&mut light_settings.ambient_reflection, 0.0..=1.0)
                .text("ambient_reflection"),
        );
        ui.add(egui::Slider::new(&mut light_settings.shininess, 1.0..=32.0).text("shininess"));
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

fn update_rotation(mut query: Query<'_, '_, (&mut Transform, &RotationComponent)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotate(Quat::from_euler(
            EulerRot::XYZ,
            rotation.rotation_speed.0 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.1 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.2 / 60.0 * std::f32::consts::PI,
        ));
    }
}

fn update_transform(
    mut renderer: ResMut<'_, Renderer>,
    mut query: Query<'_, '_, (Entity, &Transform, &mut StaticMesh)>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let mut gpu_data = renderer.aquire_transform_data();

    for (entity, transform, mut mesh) in query.iter_mut() {
        mesh.world_offset = gpu_data.ensure_index_allocated(entity.id()) as u32;

        let world = EntityTransforms {
            world: transform.compute_matrix(),
        };

        updater.add_update_jobs(&[world], u64::from(mesh.world_offset));
    }

    renderer.test_add_update_jobs(updater.job_blocks());

    renderer.release_transform_data(gpu_data);
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
        (
            &StaticMesh,
            Option<&PickedComponent>,
            Option<&ManipulatorComponent>,
        ),
    >,
    q_debug_drawables: Query<
        '_,
        '_,
        (
            &StaticMesh,
            &Transform,
            Option<&PickedComponent>,
            Option<&ManipulatorComponent>,
        ),
        Or<(With<PickedComponent>, With<ManipulatorComponent>)>,
    >,
    q_lights: Query<'_, '_, (&Transform, &LightComponent)>,
    task_pool: Res<'_, crate::RenderTaskPool>,
    mut egui: ResMut<'_, Egui>,
    mut debug_display: ResMut<'_, DebugDisplay>,
    q_cameras: Query<'_, '_, &CameraComponent>,
    light_settings: Res<'_, LightSettings>,
) {
    crate::egui::egui_plugin::end_frame(&mut egui);

    let render_context = RenderContext::new(&renderer);
    let q_drawables = q_drawables.iter().collect::<Vec<(
        &StaticMesh,
        Option<&PickedComponent>,
        Option<&ManipulatorComponent>,
    )>>();
    let q_debug_drawables = q_debug_drawables.iter().collect::<Vec<(
        &StaticMesh,
        &Transform,
        Option<&PickedComponent>,
        Option<&ManipulatorComponent>,
    )>>();

    let q_lights = q_lights
        .iter()
        .collect::<Vec<(&Transform, &LightComponent)>>();

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
            q_lights.as_slice(),
            &light_settings,
        );

        let debug_renderpass = render_surface.debug_renderpass();
        let debug_renderpass = debug_renderpass.write();
        debug_renderpass.render(
            &render_context,
            &cmd_buffer,
            render_surface.as_mut(),
            q_debug_drawables.as_slice(),
            camera_component,
            &default_meshes,
            debug_display.as_mut()            
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
