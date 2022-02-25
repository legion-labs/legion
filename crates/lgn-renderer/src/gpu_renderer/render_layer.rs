use lgn_graphics_api::{Buffer, PagedBufferAllocation};

use crate::{
    hl_gfx_api::HLCommandBuffer,
    resources::{PipelineManager, UnifiedStaticBuffer, UniformGPUDataUpdater},
};

use super::{RenderBatch, RenderElement, RenderStateSet};

pub struct RenderLayer {
    static_buffer: UnifiedStaticBuffer,
    material_page: PagedBufferAllocation,
    material_to_batch: Vec<u32>,
    batches: Vec<RenderBatch>,
    cpu_render_set: bool,
    element_count: u64,
}

impl RenderLayer {
    pub fn new(static_buffer: &UnifiedStaticBuffer, cpu_render_set: bool) -> Self {
        const TEMP_MAX_MATERIAL_COUNT: usize = 8192;
        let page_size = TEMP_MAX_MATERIAL_COUNT * std::mem::size_of::<u32>();
        let material_page = static_buffer.allocate_segment(page_size as u64);

        Self {
            static_buffer: static_buffer.clone(),
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
                .resize(material_idx as usize, batch_idx);
        } else {
            self.material_to_batch[material_idx as usize] = batch_idx;
        }
    }

    pub fn register_element(&mut self, material_idx: usize, element: &RenderElement) {
        let batch_idx = self.material_to_batch[material_idx as usize] as usize;
        if self.cpu_render_set {
            self.batches[batch_idx].add_cpu_element(element);
        } else {
            self.batches[batch_idx].add_gpu_element();
        }
        self.element_count += 1;
    }

    pub fn aggregate_offsets(&mut self, updater: &mut UniformGPUDataUpdater) {
        let aggregate_offset: u64 = 0;

        let mut per_batch_offsets = Vec::new();
        per_batch_offsets.resize(self.batches.len(), 0);

        let mut per_material_offsets = Vec::new();
        per_material_offsets.resize(self.material_to_batch.len(), 0);

        for (batch_idx, batch) in self.batches.iter().enumerate() {
            per_batch_offsets[batch_idx as usize] = aggregate_offset;

            batch.calculate_offsets(&mut aggregate_offset);
        }

        for (meterial_idx, batch_idx) in self.material_to_batch.iter().enumerate() {
            per_material_offsets[meterial_idx] = per_batch_offsets[*batch_idx as usize];
        }

        updater.add_update_jobs(&per_batch_offsets, self.material_page.offset());
    }

    pub fn get_arg_buffer_sizes(&self) -> (u64, u64) {
        (self.batches.len() as u64, self.element_count)
    }

    pub fn draw(
        &self,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        pipeline_manager: &PipelineManager,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
    ) {
        for batch in self.batches {
            batch.draw(
                cmd_buffer,
                pipeline_manager,
                indirect_arg_buffer,
                count_buffer,
            )
        }
    }
}
