use lgn_ecs::prelude::*;
use lgn_graphics_data::Color;
use lgn_math::Vec4;

use crate::{cgen, resources::UniformGPUDataUpdater};

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
    pub base_albedo: Color,
    pub base_metalness: f32,
    pub reflectance: f32,
    pub base_roughness: f32,
    pub alpha_mode: AlphaMode,
    gpu_index: u32,
    gpu_offset: u64,
}

impl Default for MaterialComponent {
    fn default() -> Self {
        Self {
            base_albedo: Color::from((204, 204, 204)),
            base_metalness: 0.0,
            reflectance: 0.5,
            base_roughness: 0.4,
            alpha_mode: AlphaMode::Opaque,
            gpu_index: u32::MAX,
            gpu_offset: u64::MAX,
        }
    }
}

impl MaterialComponent {
    pub fn gpu_offset(&self) -> u32 {
        self.gpu_offset as u32
    }

    pub(crate) fn set_gpu_material_index_offset(&mut self, index: u32, offset: u64) {
        self.gpu_index = index;
        self.gpu_offset = offset;
    }

    pub(crate) fn clear_gpu_material_index_offset(&mut self) -> u32 {
        let old_index = self.gpu_index;
        self.gpu_index = u32::MAX;
        self.gpu_offset = u64::MAX;
        old_index
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
        //gpu_material.set_alpha(self.alpha.into());

        updater.add_update_jobs(&[gpu_material], self.gpu_offset);
    }
}
