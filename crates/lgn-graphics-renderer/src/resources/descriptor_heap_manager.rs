use lgn_graphics_api::{DescriptorHeap, DescriptorHeapDef, DeviceContext};
use parking_lot::Mutex;

use super::{DescriptorPool, DescriptorPoolHandle, GpuSafePool};

pub struct DescriptorHeapManager {
    heap: DescriptorHeap,
    descriptor_pools: Mutex<GpuSafePool<DescriptorPool>>,
}

impl DescriptorHeapManager {
    pub fn new(num_render_frames: u64, device_context: &DeviceContext) -> Self {
        Self {
            heap: device_context.create_descriptor_heap(DescriptorHeapDef {
                max_descriptor_sets: 32 * 4096,
                sampler_count: 32 * 128,
                constant_buffer_count: 32 * 1024,
                buffer_count: 32 * 1024,
                rw_buffer_count: 32 * 1024,
                texture_count: 32 * 1024,
                rw_texture_count: 32 * 1024,
            }),
            descriptor_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
        }
    }

    pub fn begin_frame(&mut self, frame_index: usize) {
        let mut pool = self.descriptor_pools.lock();
        pool.begin_frame(frame_index, DescriptorPool::begin_frame);
    }

    pub fn end_frame(&mut self, frame_index: usize) {
        let mut pool = self.descriptor_pools.lock();
        pool.end_frame(frame_index, |_| ());
    }

    pub fn descriptor_heap(&self) -> &DescriptorHeap {
        &self.heap
    }

    pub fn acquire_descriptor_pool(&self, heap_def: DescriptorHeapDef) -> DescriptorPoolHandle {
        let mut pool = self.descriptor_pools.lock();
        pool.acquire_or_create(|| DescriptorPool::new(&self.heap, heap_def))
    }

    pub fn release_descriptor_pool(&self, handle: DescriptorPoolHandle) {
        let mut pool = self.descriptor_pools.lock();
        pool.release(handle);
    }
}
