use std::collections::BTreeMap;

use lgn_app::{App, Plugin};
use lgn_ecs::prelude::*;
use lgn_graphics_api::{BufferView, VertexBufferBinding};
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;

use crate::{cgen, components::VisualComponent, labels::RenderStage, Renderer};

use super::{
    IndexAllocator, StaticBufferAllocation, UnifiedStaticBufferAllocator, UniformGPUData,
    UniformGPUDataUpdater,
};

#[derive(Debug, SystemLabel, PartialEq, Eq, Clone, Copy, Hash)]
enum GpuDataPluginLabel {
    UpdateDone,
}

#[derive(Default)]
pub struct GpuDataPlugin {}

pub(crate) struct GpuDataManager<K, T> {
    gpu_data: UniformGPUData<T>,
    index_allocator: IndexAllocator,
    data_map: BTreeMap<K, Vec<(u32, u64)>>,
    default_uploaded: bool,
    default_id: u32,
    default_va: u64,
}

impl<K, T> GpuDataManager<K, T> {
    pub fn new(page_size: u64, block_size: u32) -> Self {
        let index_allocator = IndexAllocator::new(block_size);
        let gpu_data = UniformGPUData::<T>::new(None, page_size);

        Self {
            gpu_data,
            index_allocator,
            data_map: BTreeMap::new(),
            default_uploaded: false,
            default_id: u32::MAX,
            default_va: u64::MAX,
        }
    }

    pub fn alloc_gpu_data(&mut self, key: K, allocator: &UnifiedStaticBufferAllocator) -> (u32, u64)
    where
        K: Ord,
    {
        let gpu_data_id = self.index_allocator.acquire_index();
        let gpu_data_va = self.gpu_data.ensure_index_allocated(allocator, gpu_data_id);

        if let Some(gpu_data) = self.data_map.get_mut(&key) {
            gpu_data.push((gpu_data_id, gpu_data_va));
        } else {
            self.data_map.insert(key, vec![(gpu_data_id, gpu_data_va)]);
        }
        (gpu_data_id, gpu_data_va)
    }

    pub fn id_for_index(&self, optional: Option<K>, index: usize) -> u32
    where
        K: Ord,
    {
        if let Some(key) = optional {
            if let Some(value) = self.data_map.get(&key) {
                return value[index].0;
            }
        }
        self.default_id
    }

    pub fn va_for_index(&self, optional: Option<K>, index: usize) -> u64
    where
        K: Ord,
    {
        if let Some(key) = optional {
            if let Some(value) = self.data_map.get(&key) {
                return value[index].1;
            }
        }
        self.default_va
    }

    pub fn update_gpu_data(
        &self,
        key: &K,
        dest_idx: usize,
        data: &T,
        updater: &mut UniformGPUDataUpdater,
    ) where
        K: Ord,
    {
        if let Some(gpu_data) = self.data_map.get(key) {
            let data_slice = std::slice::from_ref(data);
            updater.add_update_jobs(data_slice, gpu_data[dest_idx].1);
        }
    }

    pub fn remove_gpu_data(&mut self, key: &K) -> Option<Vec<u32>>
    where
        K: Ord,
    {
        if let Some(gpu_data) = self.data_map.remove(key) {
            let mut instance_ids = Vec::with_capacity(gpu_data.len());
            for data in gpu_data {
                instance_ids.push(data.0);
            }
            self.index_allocator.release_index_ids(&instance_ids);

            Some(instance_ids)
        } else {
            None
        }
    }

    pub fn upload_default(
        &mut self,
        default: T,
        allocator: &UnifiedStaticBufferAllocator,
        updater: &mut UniformGPUDataUpdater,
    ) {
        if !self.default_uploaded {
            self.default_id = self.index_allocator.acquire_index();
            self.default_va = self
                .gpu_data
                .ensure_index_allocated(allocator, self.default_id);

            updater.add_update_jobs(&[default], self.default_va);
            self.default_uploaded = true;
        }
    }
}

pub(crate) type GpuEntityTransformManager =
    GpuDataManager<Entity, cgen::cgen_type::GpuInstanceTransform>;
pub(crate) type GpuEntityColorManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceColor>;
pub(crate) type GpuPickingDataManager =
    GpuDataManager<Entity, cgen::cgen_type::GpuInstancePickingData>;

impl Plugin for GpuDataPlugin {
    fn build(&self, app: &mut App) {
        //
        // Resources
        //
        app.insert_resource(GpuEntityTransformManager::new(64 * 1024, 1024));
        app.insert_resource(GpuEntityColorManager::new(64 * 1024, 256));
        app.insert_resource(GpuPickingDataManager::new(64 * 1024, 1024));

        //
        // Stage Prepare
        //
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(alloc_color_address)
                .with_system(alloc_transform_address)
                .label(GpuDataPluginLabel::UpdateDone),
        );
        app.add_system_set_to_stage(
            RenderStage::Prepare,
            SystemSet::new()
                .with_system(upload_transform_data)
                .after(GpuDataPluginLabel::UpdateDone),
        );
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn alloc_color_address(
    renderer: Res<'_, Renderer>,
    mut color_manager: ResMut<'_, GpuEntityColorManager>,
    query: Query<'_, '_, Entity, Added<VisualComponent>>,
) {
    for entity in query.iter() {
        color_manager.alloc_gpu_data(entity, renderer.static_buffer_allocator());
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn alloc_transform_address(
    renderer: Res<'_, Renderer>,
    mut transform_manager: ResMut<'_, GpuEntityTransformManager>,
    query: Query<'_, '_, Entity, Added<GlobalTransform>>,
) {
    for entity in query.iter() {
        transform_manager.alloc_gpu_data(entity, renderer.static_buffer_allocator());
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
fn upload_transform_data(
    renderer: Res<'_, Renderer>,
    transform_manager: Res<'_, GpuEntityTransformManager>,
    query: Query<'_, '_, (Entity, &GlobalTransform), Changed<GlobalTransform>>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);

    for (entity, transform) in query.iter() {
        let mut world = cgen::cgen_type::GpuInstanceTransform::default();
        world.set_world(transform.compute_matrix().into());

        transform_manager.update_gpu_data(&entity, 0, &world, &mut updater);
    }

    renderer.add_update_job_block(updater.job_blocks());
}

pub(crate) struct GpuVaTableForGpuInstance {
    static_allocation: StaticBufferAllocation,
}

impl GpuVaTableForGpuInstance {
    pub fn new(allocator: &UnifiedStaticBufferAllocator) -> Self {
        Self {
            static_allocation: allocator.allocate_segment(4 * 1024 * 1024),
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

    pub fn structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.static_allocation
            .structured_buffer_view(struct_size, read_only)
    }
}
