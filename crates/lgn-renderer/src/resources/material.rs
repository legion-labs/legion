use lgn_graphics_data::Color;
use lgn_math::Vec4;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::{cgen, Renderer};

use super::{IndexAllocator, UnifiedStaticBuffer, UniformGPUData, UniformGPUDataUpdater};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MaterialId(Uuid);

impl MaterialId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

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

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub material_id: MaterialId,
    pub base_color: Color,
    pub subsurface: f32,
    pub metallic: f32,
    pub specular: f32,
    pub specular_tint: f32,
    pub roughness: f32,
    pub anisotropic: f32,
    pub sheen: f32,
    pub sheen_tint: f32,
    pub clearcoat: f32,
    pub clearcoat_gloss: f32,
    pub alpha: AlphaMode,

    gpu_index: u32,
    gpu_offset: u64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            material_id: MaterialId::new(),
            base_color: Color::from((204, 204, 204)),
            subsurface: 0.0,
            metallic: 0.0,
            specular: 0.5,
            specular_tint: 0.0,
            roughness: 0.4,
            anisotropic: 0.0,
            sheen: 0.0,
            sheen_tint: 0.5,
            clearcoat: 0.0,
            clearcoat_gloss: 1.0,
            alpha: AlphaMode::Opaque,
            gpu_index: u32::MAX,
            gpu_offset: u64::MAX,
        }
    }
}

impl Material {
    pub fn gpu_offset(&self) -> u32 {
        self.gpu_offset as u32
    }

    fn set_gpu_material_index_offset(&mut self, index: u32, offset: u64) {
        self.gpu_index = index;
        self.gpu_offset = offset;
    }

    fn clear_gpu_material_index_offset(&mut self) -> u32 {
        let old_index = self.gpu_index;
        self.gpu_index = u32::MAX;
        self.gpu_offset = u64::MAX;
        old_index
    }

    fn update_gpu_data(&self, updater: &mut UniformGPUDataUpdater) {
        let mut gpu_material = cgen::cgen_type::MaterialData::default();

        let color = Vec4::new(
            f32::from(self.base_color.r) / 255.0f32,
            f32::from(self.base_color.g) / 255.0f32,
            f32::from(self.base_color.b) / 255.0f32,
            f32::from(self.base_color.a) / 255.0f32,
        );
        gpu_material.set_base_color(color.into());
        gpu_material.set_subsurface(self.subsurface.into());
        gpu_material.set_metallic(self.metallic.into());
        gpu_material.set_specular(self.specular.into());
        gpu_material.set_specular_tint(self.specular_tint.into());
        gpu_material.set_roughness(self.roughness.into());
        gpu_material.set_anisotropic(self.anisotropic.into());
        gpu_material.set_sheen(self.sheen.into());
        gpu_material.set_sheen_tint(self.sheen_tint.into());
        gpu_material.set_clearcoat(self.clearcoat.into());
        gpu_material.set_clearcoat_gloss(self.clearcoat_gloss.into());
        //gpu_material.set_alpha(self.alpha.into());

        updater.add_update_jobs(&[gpu_material], self.gpu_offset);
    }
}

pub type MaterialStaticsBuffer = UniformGPUData<cgen::cgen_type::MaterialData>;

pub struct MaterialManagerInner {
    material_map: HashMap<MaterialId, Material>,

    material_indexes: IndexAllocator,
    static_material_data: MaterialStaticsBuffer,

    new_materials: HashSet<MaterialId>,
    updated_material: HashSet<MaterialId>,
    release_gpu_indexes: Vec<u32>,
}

pub struct MaterialManager {
    inner: Arc<Mutex<MaterialManagerInner>>,
}

impl MaterialManager {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        let static_material_data = MaterialStaticsBuffer::new(static_buffer, 64 * 1024);

        Self {
            inner: Arc::new(Mutex::new(MaterialManagerInner {
                material_map: HashMap::new(),
                material_indexes: IndexAllocator::new(4096),
                static_material_data,
                new_materials: HashSet::default(),
                updated_material: HashSet::default(),
                release_gpu_indexes: Vec::new(),
            })),
        }
    }

    pub fn new_material(&self, material: Option<Material>) -> Material {
        let mut inner = self.inner.lock().unwrap();

        let mut new_material = Material::default();
        if let Some(material) = material {
            new_material = material;
        }
        assert!(inner.material_map.get(&new_material.material_id).is_none());

        inner
            .material_map
            .insert(new_material.material_id, new_material);

        inner.new_materials.insert(new_material.material_id);
        inner.updated_material.insert(new_material.material_id);

        new_material
    }

    pub fn get_material(&self, material_id: MaterialId) -> Option<Material> {
        let inner = self.inner.lock().unwrap();

        inner.material_map.get(&material_id).copied()
    }

    pub fn update_material(&self, updated_material: Material) -> bool {
        let mut inner = self.inner.lock().unwrap();

        inner.updated_material.insert(updated_material.material_id);

        if let Some(material) = inner.material_map.get_mut(&updated_material.material_id) {
            *material = updated_material;
            false
        } else {
            inner
                .material_map
                .insert(updated_material.material_id, updated_material);
            inner.new_materials.insert(updated_material.material_id);
            true
        }
    }

    pub fn remove_material(&self, material_id: MaterialId) {
        let mut inner = self.inner.lock().unwrap();

        if let Some(mut material) = inner.material_map.remove(&material_id) {
            inner
                .release_gpu_indexes
                .push(material.clear_gpu_material_index_offset());
        }
    }

    pub fn update_gpu_data(&self, renderer: &Renderer) {
        let inner = &mut *self.inner.lock().unwrap();
        let mut index_block = inner.material_indexes.acquire_index_block();

        // Remove first
        inner
            .material_indexes
            .release_index_ids(&inner.release_gpu_indexes);

        // Then newly created materials
        for new in inner.new_materials.drain() {
            if let Some(material) = inner.material_map.get_mut(&new) {
                let mut new_index = u32::MAX;
                while new_index == u32::MAX {
                    if let Some(index) = index_block.acquire_index() {
                        new_index = index;
                    } else {
                        inner.material_indexes.release_index_block(index_block);
                        index_block = inner.material_indexes.acquire_index_block();
                    }
                }

                let new_offset = inner.static_material_data.ensure_index_allocated(new_index);
                material.set_gpu_material_index_offset(new_index, new_offset);
            }
        }

        // Then all updates
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 4096 * 1024);
        for updated in inner.updated_material.drain() {
            if let Some(material) = inner.material_map.get_mut(&updated) {
                material.update_gpu_data(&mut updater);
            }
        }
        renderer.add_update_job_block(updater.job_blocks());
    }
}
