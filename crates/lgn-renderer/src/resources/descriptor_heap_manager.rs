use lgn_graphics_api::{DescriptorHeap, DescriptorHeapDef, DeviceContext};
use parking_lot::Mutex;

use super::{DescriptorPool, DescriptorPoolHandle, GpuSafePool};

pub struct DescriptorHeapManager {
    heap: DescriptorHeap,
    descriptor_pools: Mutex<GpuSafePool<DescriptorPool>>,
}

impl DescriptorHeapManager {
    pub fn new(num_render_frames: usize, device_context: &DeviceContext) -> Self {
        let descriptor_heap_def = DescriptorHeapDef {
            max_descriptor_sets: 32 * 4096,
            sampler_count: 32 * 128,
            constant_buffer_count: 32 * 1024,
            buffer_count: 32 * 1024,
            rw_buffer_count: 32 * 1024,
            texture_count: 32 * 1024,
            rw_texture_count: 32 * 1024,
        };

        Self {
            heap: device_context
                .create_descriptor_heap(&descriptor_heap_def)
                .unwrap(),
            descriptor_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
        }
    }

    pub fn begin_frame(&mut self) {
        let mut pool = self.descriptor_pools.lock();
        pool.begin_frame();
    }

    pub fn end_frame(&mut self) {
        let mut pool = self.descriptor_pools.lock();
        pool.end_frame();
    }

    pub fn descriptor_heap(&self) -> &DescriptorHeap {
        &self.heap
    }

    pub fn acquire_descriptor_pool(&self, heap_def: &DescriptorHeapDef) -> DescriptorPoolHandle {
        let mut pool = self.descriptor_pools.lock();
        pool.acquire_or_create(|| DescriptorPool::new(&self.heap, heap_def))
    }

    pub fn release_descriptor_pool(&self, handle: DescriptorPoolHandle) {
        let mut pool = self.descriptor_pools.lock();
        pool.release(handle);
    }
}
