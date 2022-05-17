use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;

use lgn_transform::components::Transform;

use crate::{
    core::{
        AsRenderObject, InsertRenderObjectCommand, RenderObjectAllocator, RenderObjectId,
        UpdateRenderObjectCommand,
    },
    lighting::RenderLight,
    Renderer,
};

#[derive(Debug, Clone, Copy)]
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
    pub render_object_id: Option<RenderObjectId>,
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            light_type: LightType::Omnidirectional,
            color: Color::WHITE,
            radiance: 40.0,
            enabled: true,
            picking_id: 0,
            render_object_id: None,
        }
    }
}

impl AsRenderObject<RenderLight> for LightComponent {
    fn as_render_object(&self) -> RenderLight {
        RenderLight {
            light_type: self.light_type,
            color: self.color,
            radiance: self.radiance,
            enabled: self.enabled,
            picking_id: self.picking_id,
        }
    }
}

#[allow(clippy::needless_pass_by_value, clippy::type_complexity)]
pub(crate) fn reflect_light_components(
    renderer: Res<'_, Renderer>,
    mut q_changes: Query<
        '_,
        '_,
        (Entity, &Transform, &mut LightComponent),
        Changed<LightComponent>,
    >,
    q_removals: RemovedComponents<'_, LightComponent>,
) {
    let mut render_commands = renderer.render_command_builder();
    renderer.allocate_render_object(|allocator: &mut RenderObjectAllocator<'_, RenderLight>| {
        for (e, xform, mut c) in q_changes.iter_mut() {
            if let Some(render_object_id) = c.render_object_id {
                render_commands.push(UpdateRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: c.as_render_object(),
                });
            } else {
                let render_object_id = allocator.alloc();
                c.render_object_id = Some(render_object_id);
                render_commands.push(InsertRenderObjectCommand::<RenderLight> {
                    render_object_id,
                    data: c.as_render_object(),
                });
            };
        }
    });

    // for e in q_removals.iter() {

    //     render_commands.push(RemoveRenderObjectCommand::<RenderLight>::new());

    //     lighting_manager
    //         .light_set()
    //         .remove(lighting_manager.render_object_id_from_entity(e));
    //}

    // for (e, transform, lightcomp) in q_changes.iter_mut() {

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
