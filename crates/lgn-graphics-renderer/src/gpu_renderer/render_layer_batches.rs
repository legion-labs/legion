use lgn_graphics_api::{Buffer, CommandBuffer, ResourceUsage};

use crate::{
    resources::{StaticBufferAllocation, UnifiedStaticBuffer},
    RenderContext,
};

use super::{GpuInstanceId, RenderBatch, RenderElement, RenderStateSet};

pub struct RenderLayerBatches {
    pub(crate) state_page: StaticBufferAllocation,
    state_to_batch: Vec<u32>,
    batches: Vec<RenderBatch>,
    cpu_render_set: bool,
    element_count: u32,
}

impl RenderLayerBatches {
    pub fn new(gpu_heap: &UnifiedStaticBuffer, cpu_render_set: bool) -> Self {
        const TEMP_MAX_MATERIAL_COUNT: usize = 8192;
        let page_size = TEMP_MAX_MATERIAL_COUNT * std::mem::size_of::<u64>();
        let state_page = gpu_heap.allocate(page_size as u64, ResourceUsage::empty());

        Self {
            state_page,
            state_to_batch: vec![],
            batches: vec![],
            cpu_render_set,
            element_count: 0,
        }
    }

    pub fn register_state_set(&mut self, state_set: &RenderStateSet) -> u32 {
        let new_index = self.batches.len() as u32;
        self.batches.push(RenderBatch::new(state_set));
        new_index
    }

    pub fn register_state(&mut self, state_id: u32, batch_idx: u32) {
        if self.state_to_batch.len() <= state_id as usize {
            self.state_to_batch
                .resize((state_id + 1) as usize, batch_idx);
        } else {
            self.state_to_batch[state_id as usize] = batch_idx;
        }
    }

    pub fn register_element(&mut self, state_id: u32, element: &RenderElement) {
        let batch_id = self.state_to_batch[state_id as usize] as usize;
        if self.cpu_render_set {
            self.batches[batch_id].add_cpu_element(element);
        } else {
            self.batches[batch_id].add_gpu_element();
        }
        self.element_count += 1;
    }

    pub fn unregister_element(&mut self, state_id: u32, gpu_instance_id: GpuInstanceId) {
        let batch_id = self.state_to_batch[state_id as usize] as usize;
        if self.cpu_render_set {
            self.batches[batch_id].remove_cpu_element(gpu_instance_id);
        } else {
            self.batches[batch_id].remove_gpu_element();
        }
        self.element_count -= 1;
    }

    pub fn aggregate_offsets(
        &mut self,
        count_buffer_offset: &mut u64,
        indirect_arg_buffer_offset: &mut u64,
    ) -> Vec<(u32, u32)> {
        if !self.cpu_render_set && !self.state_to_batch.is_empty() {
            let mut per_batch_offsets: Vec<(u32, u32)> = Vec::new();
            per_batch_offsets.resize(self.batches.len(), (0, 0));

            let mut per_state_offsets: Vec<(u32, u32)> = Vec::new();
            per_state_offsets.resize(self.state_to_batch.len(), (0, 0));

            for (batch_idx, batch) in self.batches.iter_mut().enumerate() {
                per_batch_offsets[batch_idx as usize] = (
                    *count_buffer_offset as u32,
                    *indirect_arg_buffer_offset as u32,
                );

                batch.calculate_indirect_offsets(count_buffer_offset, indirect_arg_buffer_offset);
            }

            for (state_id, batch_id) in self.state_to_batch.iter().enumerate() {
                per_state_offsets[state_id] = per_batch_offsets[*batch_id as usize];
            }

            per_state_offsets
        } else {
            vec![]
        }
    }

    pub fn offsets_va(&self) -> u64 {
        if !self.cpu_render_set && !self.state_to_batch.is_empty() {
            self.state_page.byte_offset()
        } else {
            0
        }
    }

    pub fn gpu_culling_enabled(&self) -> bool {
        !self.cpu_render_set
    }

    pub fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut CommandBuffer,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
    ) {
        for batch in &self.batches {
            batch.draw(
                render_context,
                cmd_buffer,
                indirect_arg_buffer,
                count_buffer,
            );
        }
    }
}
