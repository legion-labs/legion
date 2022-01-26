use lgn_ecs::prelude::{Changed, Query};
use std::sync::{Arc, Mutex};

use crate::components::MaterialComponent;
use crate::{cgen, Renderer};

use super::{IndexAllocator, UnifiedStaticBuffer, UniformGPUData, UniformGPUDataUpdater};

pub type MaterialStaticsBuffer = UniformGPUData<cgen::cgen_type::MaterialData>;

pub struct MaterialManagerInner {
    material_indexes: IndexAllocator,
    static_material_data: MaterialStaticsBuffer,

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
                material_indexes: IndexAllocator::new(4096),
                static_material_data,
                release_gpu_indexes: Vec::new(),
            })),
        }
    }

    pub fn remove_material(&self, material: &mut MaterialComponent) {
        let mut inner = self.inner.lock().unwrap();

        inner
            .release_gpu_indexes
            .push(material.clear_gpu_material_index_offset());
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn update_gpu_data(
        &self,
        renderer: &Renderer,
        mut updated_materials: Query<'_, '_, &mut MaterialComponent, Changed<MaterialComponent>>,
    ) {
        let inner = &mut *self.inner.lock().unwrap();
        let mut index_block = inner.material_indexes.acquire_index_block();

        // Remove first
        inner
            .material_indexes
            .release_index_ids(&inner.release_gpu_indexes);

        // Then all updates
        let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 4096 * 1024);
        for mut material in updated_materials.iter_mut() {
            if material.gpu_offset() == u32::MAX {
                let (new_index_block, new_material_id) =
                    inner.material_indexes.acquire_index(index_block);
                index_block = new_index_block;

                let new_offset = inner
                    .static_material_data
                    .ensure_index_allocated(new_material_id);
                material.set_gpu_material_index_offset(new_material_id, new_offset);
            }
            material.update_gpu_data(&mut updater);
        }
        renderer.add_update_job_block(updater.job_blocks());
    }
}
