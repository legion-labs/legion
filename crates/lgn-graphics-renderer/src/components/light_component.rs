use lgn_core::BumpAllocatorPool;
use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_math::Vec3;
use lgn_transform::components::{GlobalTransform, Transform};

use crate::{
    core::RenderObjectId, debug_display::DebugDisplay, egui::Egui, lighting::LightingManager,
    resources::DefaultMeshType, Renderer,
};

pub enum LightType {
    Omnidirectional,
    Directional,
    Spotlight { cone_angle: f32 },
}

#[derive(Component)]
pub struct LightComponent {
    pub light_type: LightType,
    pub color: Color,
    pub radiance: f32,
    pub enabled: bool,
    pub picking_id: u32,
    pub render_object_id: RenderObjectId,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Omnidirectional,
            color: Color::WHITE,
            radiance: 40.0,
            enabled: true,
            picking_id: 0,
            render_object_id: Default::default(),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_lights(
    egui: Res<'_, Egui>,
    mut lights: Query<'_, '_, &mut LightComponent>,
    mut lighting_manager: ResMut<'_, LightingManager>,
) {
    egui.window("Lights", |ui| {
        ui.checkbox(&mut lighting_manager.apply_ambient, "Apply Ambient");
        ui.checkbox(&mut lighting_manager.apply_diffuse, "Apply Diffuse");
        ui.checkbox(&mut lighting_manager.apply_specular, "Apply Specular");
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
                let mut rgb = [light.color.r, light.color.g, light.color.b];
                if ui.color_edit_button_srgb(&mut rgb).changed() {
                    light.color.r = rgb[0];
                    light.color.g = rgb[1];
                    light.color.b = rgb[2];
                }
            });
        }
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn debug_display_lights(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    lights: Query<'_, '_, (&LightComponent, &Transform)>,
) {
    if lights.is_empty() {
        return;
    }

    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for (light, transform) in lights.iter() {
                builder.add_default_mesh(
                    &GlobalTransform::identity()
                        .with_translation(transform.translation)
                        .with_scale(Vec3::new(0.2, 0.2, 0.2)) // assumes the size of sphere 1.0. Needs to be scaled in order to match picking silhouette
                        .with_rotation(transform.rotation),
                    DefaultMeshType::Sphere,
                    Color::WHITE,
                );
                match light.light_type {
                    LightType::Directional => {
                        builder.add_default_mesh(
                            &GlobalTransform::identity()
                                .with_translation(
                                    transform.translation
                                        - transform.rotation.mul_vec3(Vec3::new(0.0, 0.3, 0.0)), // assumes arrow length to be 0.3
                                )
                                .with_rotation(transform.rotation),
                            DefaultMeshType::Arrow,
                            Color::WHITE,
                        );
                    }
                    LightType::Spotlight { cone_angle, .. } => {
                        let factor = 4.0 * (cone_angle / 2.0).tan(); // assumes that default cone mesh has 1 to 4 ratio between radius and height
                        builder.add_default_mesh(
                            &GlobalTransform::identity()
                                .with_translation(
                                    transform.translation - transform.rotation.mul_vec3(Vec3::Y), // assumes cone height to be 1.0
                                )
                                .with_scale(Vec3::new(factor, 1.0, factor))
                                .with_rotation(transform.rotation),
                            DefaultMeshType::Cone,
                            Color::WHITE,
                        );
                    }
                    LightType::Omnidirectional { .. } => (),
                }
            }
        });
    });
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_lights(
    mut renderer: ResMut<'_, Renderer>,
    mut lighting_manager: ResMut<'_, LightingManager>,
    q_changes: Query<
        '_,
        '_,
        (Entity, &Transform, &LightComponent),
        Or<(Changed<Transform>, Changed<LightComponent>)>,
    >,
    removals: RemovedComponents<'_, LightComponent>,
) {
    // for e in removals.iter() {
    //     lighting_manager
    //         .light_set()
    //         .remove(lighting_manager.render_object_id_from_entity(e));
    // }

    // q_changes.for_each( |(e,t,l)| {

    // }  );

    // // TODO(vdbdd): use a stack array
    // let mut omnidirectional_lights_data =
    //     Vec::<OmniDirectionalLight>::with_capacity(OmniDirectionalLight::NUM as usize);
    // let mut directional_lights_data =
    //     Vec::<DirectionalLight>::with_capacity(DirectionalLight::NUM as usize);
    // let mut spotlights_data = Vec::<SpotLight>::with_capacity(SpotLight::NUM as usize);

    // for (transform, light) in q_lights.iter() {
    //     if !light.enabled {
    //         continue;
    //     }
    //     match light.light_type {
    //         LightType::Directional => {
    //             let direction = transform.rotation.mul_vec3(UP_VECTOR);
    //             let mut dir_light = DirectionalLight::default();
    //             dir_light.set_dir(direction.into());
    //             dir_light.set_color(u32::from(light.color).into());
    //             dir_light.set_radiance(light.radiance.into());
    //             directional_lights_data.push(dir_light);
    //         }
    //         LightType::Omnidirectional => {
    //             let mut omni_light = OmniDirectionalLight::default();
    //             omni_light.set_color(u32::from(light.color).into());
    //             omni_light.set_radiance(light.radiance.into());
    //             omni_light.set_pos(transform.translation.into());
    //             omnidirectional_lights_data.push(omni_light);
    //         }
    //         LightType::Spotlight { cone_angle } => {
    //             let direction = transform.rotation.mul_vec3(UP_VECTOR);
    //             let mut spotlight = SpotLight::default();
    //             spotlight.set_dir(direction.into());
    //             spotlight.set_color(u32::from(light.color).into());
    //             spotlight.set_radiance(light.radiance.into());
    //             spotlight.set_pos(transform.translation.into());
    //             spotlight.set_cone_angle(cone_angle.into());
    //             spotlights_data.push(spotlight);
    //         }
    //     }
    // }

    // let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    // lighting_manager.num_directional_lights = directional_lights_data.len() as u32;
    // lighting_manager.num_omnidirectional_lights = omnidirectional_lights_data.len() as u32;
    // lighting_manager.num_spotlights = spotlights_data.len() as u32;

    // if !omnidirectional_lights_data.is_empty() {
    //     let gpu_data = renderer.acquire_omnidirectional_lights_data();
    //     updater.add_update_jobs(&omnidirectional_lights_data, gpu_data.offset());
    //     renderer.release_omnidirectional_lights_data(gpu_data);
    // }
    // if !directional_lights_data.is_empty() {
    //     let gpu_data = renderer.acquire_directional_lights_data();
    //     updater.add_update_jobs(&directional_lights_data, gpu_data.offset());
    //     renderer.release_directional_lights_data(gpu_data);
    // }
    // if !spotlights_data.is_empty() {
    //     let gpu_data = renderer.acquire_spotlights_data();
    //     updater.add_update_jobs(&spotlights_data, gpu_data.offset());
    //     renderer.release_spotlights_data(gpu_data);
    // }
    // renderer.add_update_job_block(updater.job_blocks());
}
