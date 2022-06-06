use egui::Slider;
use frame_descriptor_set::FrameDescriptorSet;

use lgn_graphics_api::{BufferViewDef, ResourceUsage, TransientBufferView};
use lgn_graphics_cgen_runtime::Uint1;
use lgn_graphics_data::Color;
use lgn_math::Vec3;
use lgn_transform::prelude::GlobalTransform;

use crate::{
    cgen::{
        cgen_type::{DirectionalLight, LightingData, OmniDirectionalLight, SpotLight},
        descriptor_set::frame_descriptor_set,
    },
    components::LightType,
    core::{RenderObjectQuery, RenderObjects},
    egui::Egui,
    resources::TransientBufferAllocator,
    UP_VECTOR,
};

#[derive(Debug)]
pub struct RenderLight {
    pub transform: GlobalTransform,
    pub light_type: LightType,
    pub color: Color,
    pub radiance: f32,
    pub cone_angle: f32,
    pub enabled: bool,
    pub picking_id: u32,
}

pub struct RenderLightTestData {
    //foo: u32,
}

pub struct LightingManager {
    pub default_ambient_color: Color,
    pub default_ambient_intensity: f32,
    pub apply_diffuse: bool,
    pub apply_specular: bool,
}

trait AsGpuLight<T: Sized> {
    fn filter(&self, light_type: LightType) -> bool;
    fn as_gpu_light(&self, render_light: &RenderLight) -> T;
}

#[derive(Default)]
struct DirectionalLightWriter;

impl AsGpuLight<DirectionalLight> for DirectionalLightWriter {
    fn filter(&self, light_type: LightType) -> bool {
        light_type == LightType::Directional
    }

    fn as_gpu_light(&self, render_light: &RenderLight) -> DirectionalLight {
        let mut gpu_light = DirectionalLight::default();
        let direction = render_light.transform.rotation.mul_vec3(UP_VECTOR);
        gpu_light.set_dir(direction.into());
        gpu_light.set_color(u32::from(render_light.color).into());
        gpu_light.set_radiance(render_light.radiance.into());
        gpu_light
    }
}

#[derive(Default)]
struct SpotLightWriter;

impl AsGpuLight<SpotLight> for SpotLightWriter {
    fn filter(&self, light_type: LightType) -> bool {
        light_type == LightType::Spot
    }

    fn as_gpu_light(&self, render_light: &RenderLight) -> SpotLight {
        let mut gpu_light = SpotLight::default();

        let direction = render_light.transform.rotation.mul_vec3(UP_VECTOR);
        gpu_light.set_dir(direction.into());
        gpu_light.set_color(u32::from(render_light.color).into());
        gpu_light.set_radiance(render_light.radiance.into());
        gpu_light.set_pos(render_light.transform.translation.into());
        gpu_light.set_cone_angle(render_light.cone_angle.into());

        gpu_light
    }
}

#[derive(Default)]
struct OmniLightWriter;

impl AsGpuLight<OmniDirectionalLight> for OmniLightWriter {
    fn filter(&self, light_type: LightType) -> bool {
        light_type == LightType::OmniDirectional
    }

    fn as_gpu_light(&self, render_light: &RenderLight) -> OmniDirectionalLight {
        let mut gpu_light = OmniDirectionalLight::default();

        gpu_light.set_color(u32::from(render_light.color).into());
        gpu_light.set_radiance(render_light.radiance.into());
        gpu_light.set_pos(render_light.transform.translation.into());

        gpu_light
    }
}

impl LightingManager {
    pub fn new() -> Self {
        Self {
            default_ambient_color: Color::WHITE,
            default_ambient_intensity: 0.0,
            apply_diffuse: true,
            apply_specular: true,
        }
    }

    pub(crate) fn debug_ui(&mut self, egui: &mut Egui) {
        egui.window("Lights", |ui| {
            let mut default_ambient_color: [u8; 3] = [
                self.default_ambient_color.r,
                self.default_ambient_color.g,
                self.default_ambient_color.b,
            ];

            ui.label("Ambient");
            ui.color_edit_button_srgb(&mut default_ambient_color);
            ui.add(Slider::new(&mut self.default_ambient_intensity, 0.0..=10.0));
            ui.checkbox(&mut self.apply_diffuse, "Diffuse");
            ui.checkbox(&mut self.apply_specular, "Specular");

            self.default_ambient_color.r = default_ambient_color[0];
            self.default_ambient_color.g = default_ambient_color[1];
            self.default_ambient_color.b = default_ambient_color[2];
        });
    }

    pub(crate) fn per_frame_render(
        &self,
        render_objects: &RenderObjects,
        transient_buffer_allocator: &mut TransientBufferAllocator,
        frame_descriptor_set: &mut FrameDescriptorSet,
    ) {
        let (directional_light_buffer_view, directional_light_count) = Self::build_light_buffer(
            render_objects,
            transient_buffer_allocator,
            DirectionalLightWriter::default(),
        );

        let (spot_light_buffer_view, spot_light_count) = Self::build_light_buffer(
            render_objects,
            transient_buffer_allocator,
            SpotLightWriter::default(),
        );

        let (omni_light_buffer_view, omni_light_count) = Self::build_light_buffer(
            render_objects,
            transient_buffer_allocator,
            OmniLightWriter::default(),
        );

        let mut lighting_data = LightingData::default();

        lighting_data.set_num_directional_lights(Uint1::from(directional_light_count));
        lighting_data.set_num_omni_directional_lights(Uint1::from(omni_light_count));
        lighting_data.set_num_spot_lights(Uint1::from(spot_light_count));
        lighting_data.set_default_ambient(
            Vec3::new(
                self.default_ambient_intensity * f32::from(self.default_ambient_color.r) / 255.0,
                self.default_ambient_intensity * f32::from(self.default_ambient_color.g) / 255.0,
                self.default_ambient_intensity * f32::from(self.default_ambient_color.b) / 255.0,
            )
            .into(),
        );
        lighting_data.set_apply_diffuse(if self.apply_diffuse { 1 } else { 0 }.into());
        lighting_data.set_apply_specular(if self.apply_specular { 1 } else { 0 }.into());

        let lighting_manager_view = transient_buffer_allocator
            .copy_data(&lighting_data, ResourceUsage::AS_CONST_BUFFER)
            .to_buffer_view(BufferViewDef::as_const_buffer_typed::<LightingData>());

        frame_descriptor_set.set_lighting_data(lighting_manager_view);
        frame_descriptor_set.set_omni_directional_lights(omni_light_buffer_view);
        frame_descriptor_set.set_directional_lights(directional_light_buffer_view);
        frame_descriptor_set.set_spot_lights(spot_light_buffer_view);
    }

    #[allow(unsafe_code)]
    fn build_light_buffer<T>(
        render_objects: &RenderObjects,
        transient_buffer_allocator: &mut TransientBufferAllocator,
        writer: impl AsGpuLight<T>,
    ) -> (TransientBufferView, u32) {
        let light_query = RenderObjectQuery::<RenderLight>::new(render_objects);

        let count = light_query
            .iter()
            .filter(|render_light| render_light.enabled && writer.filter(render_light.light_type))
            .count() as u64;

        let item_count = u64::max(count, 1);

        let transient_buffer =
            transient_buffer_allocator.allocate(std::mem::size_of::<T>() as u64 * item_count);

        let ptr = transient_buffer.mapped_ptr_typed::<T>();

        let mut dst_index = 0;
        light_query.for_each(move |_, render_light| {
            if render_light.enabled && writer.filter(render_light.light_type) {
                let gpu_light = writer.as_gpu_light(render_light);
                unsafe {
                    ptr.add(dst_index).copy_from_nonoverlapping(&gpu_light, 1);
                }
                dst_index += 1;
            }
        });

        let view = transient_buffer.to_buffer_view(BufferViewDef::as_structured_buffer_typed::<T>(
            item_count, true,
        ));

        (view, count as u32)
    }
}
