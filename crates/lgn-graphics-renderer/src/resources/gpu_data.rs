use std::collections::BTreeMap;

use lgn_graphics_api::{BufferView, VertexBufferBinding};

use super::{
    IndexAllocator, StaticBufferAllocation, UnifiedStaticBufferAllocator, UniformGPUData,
    UniformGPUDataUpdater,
};

pub(crate) struct GpuDataManager<K, T> {
    gpu_data: UniformGPUData<T>,
    index_allocator: IndexAllocator,
    data_map: BTreeMap<K, Vec<(u32, u64)>>,
}

impl<K: Ord + Copy, T> GpuDataManager<K, T> {
    pub fn new(page_size: u64, block_size: u32) -> Self {
        let index_allocator = IndexAllocator::new(block_size);
        let gpu_data = UniformGPUData::<T>::new(None, page_size);

        Self {
            gpu_data,
            index_allocator,
            data_map: BTreeMap::new(),
        }
    }

    pub fn alloc_gpu_data(
        &mut self,
        key: &K,
        allocator: &UnifiedStaticBufferAllocator,
    ) -> (u32, u64) {
        let gpu_data_id = self.index_allocator.acquire_index();
        let gpu_data_va = self.gpu_data.ensure_index_allocated(allocator, gpu_data_id);

        if let Some(gpu_data) = self.data_map.get_mut(key) {
            gpu_data.push((gpu_data_id, gpu_data_va));
        } else {
            self.data_map.insert(*key, vec![(gpu_data_id, gpu_data_va)]);
        }
        (gpu_data_id, gpu_data_va)
    }

    pub fn va_for_index(&self, key: &K, index: usize) -> u64 {
        assert!(self.data_map.contains_key(key));

        let values = self.data_map.get(key).unwrap();
        values[index].1
    }

    pub fn update_gpu_data(
        &self,
        key: &K,
        dest_idx: usize,
        data: &T,
        updater: &mut UniformGPUDataUpdater,
    ) {
        assert!(self.data_map.contains_key(key));

        let gpu_data = self.data_map.get(key).unwrap();
        let data_slice = std::slice::from_ref(data);
        updater.add_update_jobs(data_slice, gpu_data[dest_idx].1);
    }

    pub fn remove_gpu_data(&mut self, key: &K) -> Option<Vec<u32>> {
        if let Some(gpu_data) = self.data_map.remove(key) {
            let mut instance_ids = Vec::with_capacity(gpu_data.len());
            for data in gpu_data {
                instance_ids.push(data.0);
            }
            self.index_allocator.release_indexes(&instance_ids);

            Some(instance_ids)
        } else {
            None
        }
    }
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
