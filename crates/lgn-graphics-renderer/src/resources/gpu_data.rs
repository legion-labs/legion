use std::collections::BTreeMap;

use super::{IndexAllocator, UnifiedStaticBufferAllocator, UniformGPUData, UniformGPUDataUpdater};

#[derive(Clone, Copy)]
pub(crate) struct GpuDataAllocation {
    index: u32,
    va_address: u64,
}

impl GpuDataAllocation {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn va_address(self) -> u64 {
        self.va_address
    }
}

pub(crate) struct GpuDataManager<K, T> {
    gpu_data: UniformGPUData<T>,
    index_allocator: IndexAllocator,
    data_map: BTreeMap<K, GpuDataAllocation>,
}

impl<K: Ord + Copy, T> GpuDataManager<K, T> {
    pub fn new(block_size: u32) -> Self {
        let index_allocator = IndexAllocator::new(block_size);
        let page_size = u64::from(block_size) * std::mem::size_of::<T>() as u64;
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
    ) -> GpuDataAllocation {
        assert!(!self.data_map.contains_key(key));

        let gpu_data_id = self.index_allocator.acquire_index();
        let gpu_data_va = self.gpu_data.ensure_index_allocated(allocator, gpu_data_id);
        let gpu_data_allocation = GpuDataAllocation {
            index: gpu_data_id,
            va_address: gpu_data_va,
        };

        self.data_map.insert(*key, gpu_data_allocation);

        gpu_data_allocation
    }

    pub fn va_for_key(&self, key: &K) -> u64 {
        assert!(self.data_map.contains_key(key));

        let values = self.data_map.get(key).unwrap();
        values.va_address
    }

    pub fn update_gpu_data(&self, key: &K, data: &T, updater: &mut UniformGPUDataUpdater) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.get(key).unwrap();
        let data_slice = std::slice::from_ref(data);
        updater.add_update_jobs(data_slice, gpu_data_allocation.va_address);
    }

    pub fn remove_gpu_data(&mut self, key: &K) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.remove(key).unwrap();
        let index_slice = std::slice::from_ref(&gpu_data_allocation.index);
        self.index_allocator.release_indexes(index_slice);
    }
}
