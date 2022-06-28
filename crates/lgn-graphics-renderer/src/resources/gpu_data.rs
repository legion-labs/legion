use std::collections::BTreeMap;

use crate::{
    core::{
        BinaryWriter, GpuUploadManager, RenderCommandBuilder, TransferError, UploadGPUBuffer,
        UploadGPUResource,
    },
    resources::UpdateUnifiedStaticBufferCommand,
};

use super::{IndexAllocator, UnifiedStaticBuffer, UniformGPUData};

#[derive(Clone, Copy)]
pub(crate) struct GpuDataAllocation {
    index: u32,
    gpuheap_addr: u64,
}

impl GpuDataAllocation {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn gpuheap_addr(self) -> u64 {
        self.gpuheap_addr
    }
}

pub(crate) struct GpuDataManager<K, T> {
    gpu_heap: UnifiedStaticBuffer,
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
        let gpu_data = UniformGPUData::<T>::new(gpu_heap, page_size);

        Self {
            gpu_heap: gpu_heap.clone(),
            gpu_data,
            index_allocator,
            data_map: BTreeMap::new(),
            gpu_upload_manager: gpu_upload_manager.clone(),
        }
    }

    pub fn alloc_gpu_data(&mut self, key: &K) -> GpuDataAllocation {
        assert!(!self.data_map.contains_key(key));

        let gpu_data_id = self.index_allocator.allocate();
        let gpuheap_addr = self.gpu_data.ensure_index_allocated(gpu_data_id);
        let gpu_data_allocation = GpuDataAllocation {
            index: gpu_data_id,
            gpuheap_addr,
        };

        self.data_map.insert(*key, gpu_data_allocation);

        gpu_data_allocation
    }

    pub fn gpuheap_addr_for_key(&self, key: &K) -> u64 {
        assert!(self.data_map.contains_key(key));

        let values = self.data_map.get(key).unwrap();
        values.gpuheap_addr
    }

    pub fn update_gpu_data(&self, key: &K, data: &T, render_commands: &mut RenderCommandBuilder) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.get(key).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(data);

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: binary_writer.take(),
            dst_offset: gpu_data_allocation.gpuheap_addr,
        });
    }

    pub fn sync_update_gpu_data(&self, key: &K, data: &T) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.get(key).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(data);

        self.gpu_upload_manager
            .push(UploadGPUResource::Buffer(UploadGPUBuffer {
                src_data: binary_writer.take(),
                dst_buffer: self.gpu_heap.buffer().clone(),
                dst_offset: gpu_data_allocation.gpuheap_addr,
            }));
    }

    pub async fn async_update_gpu_data(&self, key: &K, data: &T) -> Result<(), TransferError> {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.get(key).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(data);

        self.gpu_upload_manager
            .async_upload(UploadGPUResource::Buffer(UploadGPUBuffer {
                src_data: binary_writer.take(),
                dst_buffer: self.gpu_heap.buffer().clone(),
                dst_offset: gpu_data_allocation.gpuheap_addr,
            }))?;

        Ok(())
    }

    pub fn remove_gpu_data(&mut self, key: &K) {
        assert!(self.data_map.contains_key(key));

        let gpu_data_allocation = self.data_map.remove(key).unwrap();
        self.index_allocator.free(gpu_data_allocation.index);
    }
}
