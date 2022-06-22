use std::collections::BTreeMap;

use crate::{
    core::{
        BinaryWriter, GpuUploadManager, RenderCommandBuilder, UploadGPUBuffer, UploadGPUResource,
    },
    resources::UpdateUnifiedStaticBufferCommand,
};

use super::{IndexAllocator, UnifiedStaticBufferAllocator, UniformGPUData, UnifiedStaticBuffer};

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
    gpu_upload_manager: GpuUploadManager,
}

impl<K: Ord + Copy, T> GpuDataManager<K, T> {
    pub fn new(
        gpu_heap: &UnifiedStaticBuffer,
        block_size: u32,
        gpu_upload_manager: &GpuUploadManager,
    ) -> Self {
        let index_allocator = IndexAllocator::new(block_size);
        let page_size = u64::from(block_size) * std::mem::size_of::<T>() as u64;
        let gpu_data = UniformGPUData::<T>::new(allocator, page_size);
        

        Self {
            gpu_data,
            index_allocator,
            data_map: BTreeMap::new(),
            gpu_upload_manager: gpu_upload_manager.clone(),
        }
    }

    pub fn alloc_gpu_data(&mut self, key: &K) -> GpuDataAllocation {
        assert!(!self.data_map.contains_key(key));

        let gpu_data_id = self.index_allocator.allocate();
        let gpu_data_va = self.gpu_data.ensure_index_allocated(gpu_data_id);
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

    pub fn update_gpu_data(&self, key: &K, data: &T, render_commands: &mut RenderCommandBuilder) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.get(key).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(data);

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: binary_writer.take(),
            dst_offset: gpu_data_allocation.va_address,
        });
    }

    pub async fn async_update_gpu_data(&self, key: &K, data: &T) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.get(key).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(data);

        self.gpu_upload_manager
            .async_upload(UploadGPUResource::Buffer(UploadGPUBuffer {
                src_data: binary_writer.take(),
                dst_buffer: self.,
                dst_offset: gpu_data_allocation.va_address,
            }))

        // render_commands.push(UpdateUnifiedStaticBufferCommand {
        //     src_buffer: binary_writer.take(),
        //     dst_offset: gpu_data_allocation.va_address,
        // });
    }

    pub fn remove_gpu_data(&mut self, key: &K) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.remove(key).unwrap();
        self.index_allocator.free(gpu_data_allocation.index);
    }
}
