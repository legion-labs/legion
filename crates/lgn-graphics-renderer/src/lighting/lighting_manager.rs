use frame_descriptor_set::FrameDescriptorSet;

use lgn_graphics_api::{
    Buffer, BufferCreateFlags, BufferDef, BufferView, BufferViewDef, DeviceContext, MemoryUsage,
    ResourceUsage,
};

use crate::{
    cgen::{
        cgen_type::{DirectionalLight, LightingData, OmniDirectionalLight, SpotLight},
        descriptor_set::frame_descriptor_set,
    },
    core::RenderObjectSet,
    resources::TransientBufferAllocator,
};

const OMNI_DIRECTIONAL_LIGHT_MAX: u64 = 4096;
const DIRECTIONAL_LIGHT_MAX: u64 = 4096;
const SPOT_LIGHT_MAX: u64 = 4096;

// macro_rules! impl_static_buffer_accessor {
//     ($name:ident, $buffer_type:ty, $type:ty) => {
//         paste::paste! {
//             pub fn [<acquire_ $name>](&mut self) -> $buffer_type {
//                 self.$name.transfer()
//             }
//             pub fn [<release_ $name>](&mut self, $name: $buffer_type) {
//                 self.$name = $name;
//             }
//             pub fn [<$name _create_structured_buffer_view>](&self) -> BufferView {
//                 self.$name.create_structured_buffer_view($type::NUM)
//             }
//         }
//     };
// }

pub struct RenderLight {}

type RenderObjectLights = RenderObjectSet<RenderLight>;

pub struct LightingManager {
    pub render_object_lights: RenderObjectLights,

    pub omnidirectional_light_buffer: Buffer,
    pub omnidirectional_light_bufferview: BufferView,
    pub directional_light_buffer: Buffer,
    pub directional_light_bufferview: BufferView,
    pub spot_light_buffer: Buffer,
    pub spot_light_bufferview: BufferView,

    pub num_directional_lights: u32,
    pub num_omnidirectional_lights: u32,
    pub num_spotlights: u32,

    pub apply_ambient: bool,
    pub apply_diffuse: bool,
    pub apply_specular: bool,
}

impl LightingManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        let omnidirectional_lights_buffer = device_context.create_buffer(
            BufferDef {
                size: std::mem::size_of::<OmniDirectionalLight>() as u64
                    * OMNI_DIRECTIONAL_LIGHT_MAX,
                usage_flags: ResourceUsage::AS_SHADER_RESOURCE,
                create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::CpuToGpu,
                always_mapped: true,
            },
            "omni_directional_lights",
        );
        let omnidirectional_lights_bufferview =
            omnidirectional_lights_buffer.create_view(BufferViewDef::as_structured_buffer_typed::<
                OmniDirectionalLight,
            >(
                OMNI_DIRECTIONAL_LIGHT_MAX, true
            ));

        let directional_lights_buffer = device_context.create_buffer(
            BufferDef {
                size: std::mem::size_of::<DirectionalLight>() as u64 * DIRECTIONAL_LIGHT_MAX,
                usage_flags: ResourceUsage::AS_SHADER_RESOURCE,
                create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::CpuToGpu,
                always_mapped: true,
            },
            "directional_lights",
        );
        let directional_lights_bufferview =
            directional_lights_buffer.create_view(BufferViewDef::as_structured_buffer_typed::<
                DirectionalLight,
            >(DIRECTIONAL_LIGHT_MAX, true));

        let spot_lights_buffer = device_context.create_buffer(
            BufferDef {
                size: std::mem::size_of::<SpotLight>() as u64 * SPOT_LIGHT_MAX,
                usage_flags: ResourceUsage::AS_SHADER_RESOURCE,
                create_flags: BufferCreateFlags::empty(),
                memory_usage: MemoryUsage::CpuToGpu,
                always_mapped: true,
            },
            "spot_lights",
        );
        let spotlights_bufferview = spot_lights_buffer.create_view(
            BufferViewDef::as_structured_buffer_typed::<SpotLight>(SPOT_LIGHT_MAX, true),
        );

        Self {
            render_object_lights: RenderObjectLights::new(256),

            omnidirectional_light_buffer: omnidirectional_lights_buffer,
            omnidirectional_light_bufferview: omnidirectional_lights_bufferview,
            directional_light_buffer: directional_lights_buffer,
            directional_light_bufferview: directional_lights_bufferview,
            spot_light_buffer: spot_lights_buffer,
            spot_light_bufferview: spotlights_bufferview,

            num_directional_lights: 0,
            num_omnidirectional_lights: 0,
            num_spotlights: 0,

            apply_ambient: true,
            apply_diffuse: true,
            apply_specular: true,
        }
    }

    fn gpu_data(&self) -> LightingData {
        let mut lighting_data = LightingData::default();

        lighting_data.set_num_directional_lights(self.num_directional_lights.into());
        lighting_data.set_num_omni_directional_lights(self.num_omnidirectional_lights.into());
        lighting_data.set_num_spot_lights(self.num_spotlights.into());
        lighting_data.set_apply_ambient(if self.apply_ambient { 1 } else { 0 }.into());
        lighting_data.set_apply_diffuse(if self.apply_diffuse { 1 } else { 0 }.into());
        lighting_data.set_apply_specular(if self.apply_specular { 1 } else { 0 }.into());

        lighting_data
    }

    pub fn per_frame_prepare(
        &mut self,
        // mut renderer: ResMut<'_, Renderer>,
        // q_lights: Query<'_, '_, (&Transform, &LightComponent)>,
        // q_change: Query<'_, '_, &LightComponent, Or<(Changed<LightComponent>, Changed<Transform>)>>,
        // removals: RemovedComponents<'_, LightComponent>,
    ) {
        // self.render_object_lights.added();

        // self.render_object_lights.changed();

        // self.render_object_lights.removed();

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

    pub(crate) fn per_frame_render(
        &self,
        transient_buffer_allocator: &mut TransientBufferAllocator,
        frame_descriptor_set: &mut FrameDescriptorSet,
    ) {
        let a =
            transient_buffer_allocator.copy_data(&self.gpu_data(), ResourceUsage::AS_CONST_BUFFER);

        let lighting_manager_view =
            a.to_buffer_view(BufferViewDef::as_const_buffer_typed::<LightingData>());

        frame_descriptor_set.set_lighting_data(lighting_manager_view);
        frame_descriptor_set.set_omni_directional_lights(&self.omnidirectional_light_bufferview);
        frame_descriptor_set.set_directional_lights(&self.directional_light_bufferview);
        frame_descriptor_set.set_spot_lights(&self.spot_light_bufferview);
    }
}
