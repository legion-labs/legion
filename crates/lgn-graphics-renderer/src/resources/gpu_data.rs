use std::collections::BTreeMap;

use lgn_app::{App, Plugin};
use lgn_ecs::prelude::{Commands, Entity};
use lgn_graphics_api::{PagedBufferAllocation, VertexBufferBinding};

use crate::{cgen, components::MaterialComponent};

use super::{
    IndexAllocator, IndexBlock, UnifiedStaticBuffer, UniformGPUData, UniformGPUDataUpdater,
};

pub struct GpuDataPlugin {
    static_buffer: UnifiedStaticBuffer,
}

impl GpuDataPlugin {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            static_buffer: static_buffer.clone(),
        }
    }
}

pub(crate) struct GpuDataManager<T> {
    gpu_data: UniformGPUData<T>,
    index_allocator: IndexAllocator,
    entity_data_hash: BTreeMap<Entity, Vec<(u32, u64)>>,
}

impl<T> GpuDataManager<T> {
    pub fn new(static_buffer: &UnifiedStaticBuffer, page_size: u64, block_size: u32) -> Self {
        Self {
            gpu_data: UniformGPUData::<T>::new(static_buffer, page_size),
            index_allocator: IndexAllocator::new(block_size),
            entity_data_hash: BTreeMap::new(),
        }
    }

    pub fn alloc_gpu_data(
        &mut self,
        entity: Entity,
        index_block: &mut Option<IndexBlock>,
    ) -> (u32, u64) {
        let gpu_data_id = self.index_allocator.acquire_index(index_block);
        let gpu_data_va = self.gpu_data.ensure_index_allocated(gpu_data_id);

        if let Some(gpu_data) = self.entity_data_hash.get_mut(&entity) {
            gpu_data.push((gpu_data_id, gpu_data_va));
        } else {
            self.entity_data_hash
                .insert(entity, vec![(gpu_data_id, gpu_data_va)]);
        }
        (gpu_data_id, gpu_data_va)
    }

    pub fn id_va_list(&self, entity: Entity) -> &[(u32, u64)] {
        self.entity_data_hash.get(&entity).unwrap()
    }

    pub fn update_gpu_data(
        &self,
        entity: Entity,
        dest_idx: usize,
        data: &[T],
        updater: &mut UniformGPUDataUpdater,
    ) {
        if let Some(gpu_data) = self.entity_data_hash.get(&entity) {
            updater.add_update_jobs(data, gpu_data[dest_idx].1);
        }
    }

    pub fn remove_gpu_data(&mut self, entity: Entity) {
        if let Some(gpu_data) = self.entity_data_hash.remove(&entity) {
            let mut instance_ids = Vec::with_capacity(gpu_data.len());
            for data in gpu_data {
                instance_ids.push(data.0);
            }
            self.index_allocator.release_index_ids(&instance_ids);
        }
    }

    pub fn return_index_block(&self, index_block: Option<IndexBlock>) {
        if let Some(block) = index_block {
            self.index_allocator.release_index_block(block);
        }
    }
}

pub(crate) type GpuEntityTransformManager = GpuDataManager<cgen::cgen_type::GpuInstanceTransform>;
pub(crate) type GpuEntityColorManager = GpuDataManager<cgen::cgen_type::GpuInstanceColor>;
pub(crate) type GpuPickingDataManager = GpuDataManager<cgen::cgen_type::GpuInstancePickingData>;

pub(crate) type GpuMaterialData = UniformGPUData<cgen::cgen_type::MaterialData>;

pub struct GpuUniformData {
    pub gpu_texture_id_allocator: IndexAllocator,
    pub gpu_material_id_allocator: IndexAllocator,
    pub gpu_material_data: GpuMaterialData,
    pub default_material_gpu_offset: u32,
}

impl GpuUniformData {
    fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            gpu_texture_id_allocator: IndexAllocator::new(256),
            gpu_material_id_allocator: IndexAllocator::new(256),
            gpu_material_data: GpuMaterialData::new(static_buffer, 64 * 1024),
            default_material_gpu_offset: u32::MAX,
        }
    }

    pub fn initialize_default_material(&mut self, mut commands: Commands<'_, '_>) {
        let mut data_context = GpuUniformDataContext::new(self);
        let default = MaterialComponent::new(&mut data_context);
        std::mem::drop(data_context);

        self.default_material_gpu_offset = default.gpu_offset();
        commands.spawn().insert(default);
    }
}

pub struct GpuUniformDataContext<'a> {
    pub uniform_data: &'a GpuUniformData,

    pub gpu_texture_id_block: Option<IndexBlock>,
    pub gpu_material_id_block: Option<IndexBlock>,
}

impl<'a> Drop for GpuUniformDataContext<'a> {
    fn drop(&mut self) {
        if let Some(index_block) = self.gpu_texture_id_block.take() {
            self.uniform_data
                .gpu_texture_id_allocator
                .release_index_block(index_block);
        }
        if let Some(index_block) = self.gpu_material_id_block.take() {
            self.uniform_data
                .gpu_material_id_allocator
                .release_index_block(index_block);
        }
    }
}

impl<'a> GpuUniformDataContext<'a> {
    pub fn new(uniform_data: &'a GpuUniformData) -> Self {
        Self {
            uniform_data,
            gpu_texture_id_block: None,
            gpu_material_id_block: None,
        }
    }

    pub fn aquire_gpu_texture_id(&mut self) -> u32 {
        self.uniform_data
            .gpu_texture_id_allocator
            .acquire_index(&mut self.gpu_texture_id_block)
    }

    pub fn aquire_gpu_material_id(&mut self) -> u32 {
        self.uniform_data
            .gpu_material_id_allocator
            .acquire_index(&mut self.gpu_material_id_block)
    }
}

impl Plugin for GpuDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GpuUniformData::new(&self.static_buffer));
        app.insert_resource(GpuEntityTransformManager::new(
            &self.static_buffer,
            64 * 1024,
            1024,
        ));
        app.insert_resource(GpuEntityColorManager::new(
            &self.static_buffer,
            64 * 1024,
            256,
        ));
        app.insert_resource(GpuPickingDataManager::new(
            &self.static_buffer,
            64 * 1024,
            1024,
        ));
    }
}

pub(crate) struct GpuVaTableForGpuInstance {
    static_allocation: PagedBufferAllocation,
}

impl GpuVaTableForGpuInstance {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            static_allocation: static_buffer.allocate_segment(4 * 1024 * 1024),
        }
    }

    pub fn set_va_table_address_for_gpu_instance(
        &self,
        updater: &mut UniformGPUDataUpdater,
        gpu_instance: u32,
        va_table_address: u32,
    ) {
        let offset_for_gpu_instance = self.static_allocation.offset() + u64::from(gpu_instance) * 4;

        updater.add_update_jobs(&[va_table_address], offset_for_gpu_instance);
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.static_allocation.vertex_buffer_binding()
    }
}
