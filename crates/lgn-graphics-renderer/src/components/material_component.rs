use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_math::Vec4;

use crate::{
    cgen,
    resources::{GpuUniformDataContext, UniformGPUDataUpdater},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask(f32),
    Blend(f32),
}

impl Eq for AlphaMode {}

impl Default for AlphaMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct MaterialComponent {
    pub albedo_texture: u32,
    pub base_albedo: Color,
    pub normal_texture: u32,
    pub metalness_texture: u32,
    pub base_metalness: f32,
    pub reflectance: f32,
    pub roughness_texture: u32,
    pub base_roughness: f32,
    pub alpha_mode: AlphaMode,
    gpu_index: u32,
    gpu_offset: u64,
}

impl MaterialComponent {
    pub fn new(data_context: &mut GpuUniformDataContext<'_>) -> Self {
        let gpu_index = data_context.aquire_gpu_material_id();
        let gpu_offset = data_context
            .uniform_data
            .gpu_material_data
            .ensure_index_allocated(gpu_index);

        Self {
            albedo_texture: u32::MAX,
            base_albedo: Color::from((204, 204, 204)),
            normal_texture: u32::MAX,
            metalness_texture: u32::MAX,
            base_metalness: 0.0,
            reflectance: 0.5,
            roughness_texture: u32::MAX,
            base_roughness: 0.4,
            alpha_mode: AlphaMode::Opaque,
            gpu_index,
            gpu_offset,
        }
    }

    pub fn gpu_index(&self) -> u32 {
        self.gpu_index
    }

    pub fn gpu_offset(&self) -> u32 {
        self.gpu_offset as u32
    }

    pub(crate) fn update_gpu_data(&self, updater: &mut UniformGPUDataUpdater) {
        let mut gpu_material = cgen::cgen_type::MaterialData::default();

        let color = Vec4::new(
            f32::from(self.base_albedo.r) / 255.0f32,
            f32::from(self.base_albedo.g) / 255.0f32,
            f32::from(self.base_albedo.b) / 255.0f32,
            f32::from(self.base_albedo.a) / 255.0f32,
        );
        gpu_material.set_base_albedo(color.into());
        gpu_material.set_base_metalness(self.base_metalness.into());
        gpu_material.set_reflectance(self.reflectance.into());
        gpu_material.set_base_roughness(self.base_roughness.into());
        gpu_material.set_albedo_texture(self.albedo_texture.into());
        gpu_material.set_normal_texture(self.normal_texture.into());
        gpu_material.set_metalness_texture(self.metalness_texture.into());
        gpu_material.set_roughness_texture(self.roughness_texture.into());
        //gpu_material.set_alpha(self.alpha.into());

        updater.add_update_jobs(&[gpu_material], self.gpu_offset);
    }
}
