use lgn_graphics_api::Buffer;

use crate::{
    hl_gfx_api::HLCommandBuffer,
    resources::{StaticBufferAllocation, UnifiedStaticBuffer, UniformGPUDataUpdater},
    RenderContext,
};

use super::{RenderBatch, RenderElement, RenderStateSet};

pub struct RenderLayer {
    material_page: StaticBufferAllocation,
    material_to_batch: Vec<u32>,
    batches: Vec<RenderBatch>,
    cpu_render_set: bool,
    element_count: u32,
}

impl RenderLayer {
    pub fn new(static_buffer: &UnifiedStaticBuffer, cpu_render_set: bool) -> Self {
        const TEMP_MAX_MATERIAL_COUNT: usize = 8192;
        let page_size = TEMP_MAX_MATERIAL_COUNT * std::mem::size_of::<u32>();
        let material_page = static_buffer.allocate_segment(page_size as u64);

        Self {
            material_page,
            material_to_batch: vec![],
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

    pub fn register_material(&mut self, material_idx: u32, batch_idx: u32) {
        if self.material_to_batch.len() < material_idx as usize {
            self.material_to_batch
                .resize((material_idx + 1) as usize, batch_idx);
        } else {
            self.material_to_batch[material_idx as usize] = batch_idx;
        }
    }

    pub fn register_element(&mut self, material_idx: u32, element: &RenderElement) {
        let batch_idx = self.material_to_batch[material_idx as usize] as usize;
        if self.cpu_render_set {
            self.batches[batch_idx].add_cpu_element(element);
        } else {
            self.batches[batch_idx].add_gpu_element();
        }
        self.element_count += 1;
    }

    pub fn unregister_element(&mut self, material_idx: u32, gpu_instance_id: u32) {
        let batch_idx = self.material_to_batch[material_idx as usize] as usize;
        if self.cpu_render_set {
            self.batches[batch_idx].remove_cpu_element(gpu_instance_id);
        } else {
            self.batches[batch_idx].remove_gpu_element();
        }
        self.element_count -= 1;
    }

    pub fn aggregate_offsets(
        &mut self,
        updater: &mut UniformGPUDataUpdater,
        count_buffer_offset: &mut u64,
        indirect_arg_buffer_offset: &mut u64,
    ) -> u32 {
        if !self.cpu_render_set && !self.material_to_batch.is_empty() {
            let mut per_batch_offsets: Vec<(u32, u32)> = Vec::new();
            per_batch_offsets.resize(self.batches.len(), (0, 0));

            let mut per_material_offsets: Vec<(u32, u32)> = Vec::new();
            per_material_offsets.resize(self.material_to_batch.len(), (0, 0));

            for (batch_idx, batch) in self.batches.iter_mut().enumerate() {
                per_batch_offsets[batch_idx as usize] = (
                    *count_buffer_offset as u32,
                    *indirect_arg_buffer_offset as u32,
                );

                *count_buffer_offset += 1;
                batch.calculate_indirect_offsets(indirect_arg_buffer_offset);
            }

            for (material_id, batch_id) in self.material_to_batch.iter().enumerate() {
                per_material_offsets[material_id] = per_batch_offsets[*batch_id as usize];
            }

            updater.add_update_jobs(&per_material_offsets, self.material_page.offset());

            self.material_page.offset() as u32
        } else {
            0
        }
    }

    pub fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
    ) {
        let mut count_offset = 0;
        for batch in &self.batches {
            batch.draw(
                render_context,
                cmd_buffer,
                indirect_arg_buffer,
                count_buffer,
                count_offset,
            );
            count_offset += 4;
        }
    }
}
