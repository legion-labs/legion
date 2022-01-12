#![allow(unsafe_code)]

use lgn_ecs::prelude::*;
use lgn_math::Vec3;
use lgn_transform::components::Transform;

use crate::{
    cgen::cgen_type::{DirectionalLight, OmnidirectionalLight, Spotlight},
    debug_display::DebugDisplay,
    egui::egui_plugin::Egui,
    lighting::LightingManager,
    resources::{DefaultMeshId, UniformGPUDataUpdater},
    Renderer,
};

pub enum LightType {
    Omnidirectional,
    Directional,
    Spotlight { cone_angle: f32 },
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: (f32, f32, f32),
    pub radiance: f32,
    pub enabled: bool,
    pub picking_id: u32,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Omnidirectional,
            color: (1.0, 1.0, 1.0),
            radiance: 40.0,
            enabled: true,
            picking_id: 0,
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_lights(
    egui_ctx: Res<'_, Egui>,
    mut lights: Query<'_, '_, &mut LightComponent>,
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
        for (i, mut light) in lights.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::Checkbox::new(&mut light.enabled, "Enabled"));
                match light.light_type {
                    LightType::Directional => {
                        ui.label(format!("Directional light {}: ", i));
                    }
                    LightType::Omnidirectional { .. } => {
                        ui.label(format!("Omni light {}: ", i));
                    }
                    LightType::Spotlight { ref mut cone_angle } => {
                        ui.label(format!("Spotlight {}: ", i));
                        ui.add(
                            egui::Slider::new(cone_angle, -0.0..=std::f32::consts::PI)
                                .text("angle"),
                        );
                    }
                }
                ui.add(egui::Slider::new(&mut light.radiance, 0.0..=300.0).text("radiance"));
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn debug_display_lights(
    renderer: Res<'_, Renderer>,
    mut debug_display: ResMut<'_, DebugDisplay>,
    lights: Query<'_, '_, (&LightComponent, &Transform)>,
) {
    let bump = renderer.acquire_bump_allocator();
    debug_display.create_display_list(bump.bump(), |display_list| {
        for (light, transform) in lights.iter() {
            display_list.add_mesh(
                Transform::identity()
                    .with_translation(transform.translation)
                    .with_scale(transform.scale * 0.2) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                    .with_rotation(transform.rotation)
                    .compute_matrix(),
                DefaultMeshId::Sphere as u32,
                light.color,
                bump.bump(),
            );
            match light.light_type {
                LightType::Directional => {
                    display_list.add_mesh(
                        Transform::identity()
                            .with_translation(
                                transform.translation
                                    - transform.rotation.mul_vec3(Vec3::new(0.0, 0.3, 0.0)), // assumes arrow length to be 0.3
                            )
                            .with_scale(transform.scale)
                            .with_rotation(transform.rotation)
                            .compute_matrix(),
                        DefaultMeshId::Arrow as u32,
                        light.color,
                        bump.bump(),
                    );
                }
                LightType::Spotlight { cone_angle, .. } => {
                    let factor = 4.0 * (cone_angle / 2.0).tan(); // assumes that default cone mesh has 1 to 4 ratio between radius and height
                    display_list.add_mesh(
                        Transform::identity()
                            .with_translation(
                                transform.translation
                                    - transform.rotation.mul_vec3(Vec3::new(0.0, 1.0, 0.0)), // assumes cone height to be 1.0
                            )
                            .with_scale(transform.scale * Vec3::new(factor, 1.0, factor))
                            .with_rotation(transform.rotation)
                            .compute_matrix(),
                        DefaultMeshId::Cone as u32,
                        light.color,
                        bump.bump(),
                    );
                }
                LightType::Omnidirectional { .. } => (),
            }
        }
    });
    renderer.release_bump_allocator(bump);
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn update_lights(
    mut renderer: ResMut<'_, Renderer>,
    mut lighting_manager: ResMut<'_, LightingManager>,
    q_lights: Query<'_, '_, (&Transform, &LightComponent)>,
    q_change: Query<'_, '_, &LightComponent, Or<(Changed<LightComponent>, Changed<Transform>)>>,
    removals: RemovedComponents<'_, LightComponent>,
) {
    if q_change.is_empty() && removals.iter().count() == 0 {
        return;
    }
    let mut omnidirectional_lights_data =
        Vec::<OmnidirectionalLight>::with_capacity(OmnidirectionalLight::NUM as usize);
    let mut directional_lights_data =
        Vec::<DirectionalLight>::with_capacity(DirectionalLight::NUM as usize);
    let mut spotlights_data = Vec::<Spotlight>::with_capacity(Spotlight::NUM as usize);

    for (transform, light) in q_lights.iter() {
        if !light.enabled {
            continue;
        }
        match light.light_type {
            LightType::Directional => {
                let direction = transform.rotation.mul_vec3(Vec3::Y);
                let mut dir_light = DirectionalLight::default();
                dir_light.set_dir(direction.into());
                dir_light.set_color(Vec3::new(light.color.0, light.color.1, light.color.2).into());
                dir_light.set_radiance(light.radiance.into());
                directional_lights_data.push(dir_light);
            }
            LightType::Omnidirectional => {
                let mut omni_light = OmnidirectionalLight::default();
                omni_light.set_color(Vec3::new(light.color.0, light.color.1, light.color.2).into());
                omni_light.set_radiance(light.radiance.into());
                omni_light.set_pos(transform.translation.into());
                omnidirectional_lights_data.push(omni_light);
            }
            LightType::Spotlight { cone_angle } => {
                let direction = transform.rotation.mul_vec3(Vec3::Y);
                let mut spotlight = Spotlight::default();
                spotlight.set_dir(direction.into());
                spotlight.set_color(Vec3::new(light.color.0, light.color.1, light.color.2).into());
                spotlight.set_radiance(light.radiance.into());
                spotlight.set_pos(transform.translation.into());
                spotlight.set_cone_angle(cone_angle.into());
                spotlights_data.push(spotlight);
            }
        }
    }

    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    lighting_manager.num_directional_lights = directional_lights_data.len() as u32;
    lighting_manager.num_omnidirectional_lights = omnidirectional_lights_data.len() as u32;
    lighting_manager.num_spotlights = spotlights_data.len() as u32;

    if !omnidirectional_lights_data.is_empty() {
        let gpu_data = renderer.acquire_omnidirectional_lights_data();
        updater.add_update_jobs(&omnidirectional_lights_data, gpu_data.offset());
        renderer.release_omnidirectional_lights_data(gpu_data);
    }
    if !directional_lights_data.is_empty() {
        let gpu_data = renderer.acquire_directional_lights_data();
        updater.add_update_jobs(&directional_lights_data, gpu_data.offset());
        renderer.release_directional_lights_data(gpu_data);
    }
    if !spotlights_data.is_empty() {
        let gpu_data = renderer.acquire_spotlights_data();
        updater.add_update_jobs(&spotlights_data, gpu_data.offset());
        renderer.release_spotlights_data(gpu_data);
    }
    renderer.test_add_update_jobs(updater.job_blocks());
}
