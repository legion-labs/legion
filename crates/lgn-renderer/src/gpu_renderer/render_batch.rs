use lgn_graphics_api::Buffer;

use crate::{
    hl_gfx_api::HLCommandBuffer,
    resources::{PipelineHandle, PipelineManager},
};

use super::RenderElement;

#[derive(Clone)]
pub(crate) struct RenderStateSet {
    pub(super) pipeline_handle: PipelineHandle,
}

#[derive(Clone)]
pub struct RenderBatch {
    state_set: RenderStateSet,
    elements: Vec<RenderElement>,
    element_count: u32,
    element_offset: u64,
}

impl RenderBatch {
    pub fn new(state_set: &RenderStateSet) -> Self {
        Self {
            state_set: state_set.clone(),
            elements: vec![],
            element_count: 0,
            element_offset: u64::MAX,
        }
    }

    pub fn add_cpu_element(&mut self, element: &RenderElement) {
        self.elements.push(*element);
    }

    pub fn remove_cpu_element(&mut self, element: &RenderElement) {
        for (index, matching) in self.elements.iter().enumerate() {
            if element.gpu_instance_id == matching.gpu_instance_id {
                self.elements.swap_remove(index);
                return;
            }
        }
    }

    pub fn reset_cpu_elements(&mut self) {
        self.elements.clear();
    }

    pub fn add_gpu_element(&mut self) {
        self.element_count += 1;
    }

    pub fn remove_gpu_element(&mut self) {
        self.element_count -= 1;
    }

    pub fn calculate_offsets(&mut self, aggregate_offset: &mut u64) {
        self.element_offset = *aggregate_offset;
        *aggregate_offset += self.element_count as u64;
    }

    pub fn draw(
        &self,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        pipeline_manager: &PipelineManager,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
    ) {
        const INDIRECT_ARG_STRIDE: u32 = 20;

        let pipeline = pipeline_manager
            .get_pipeline(self.state_set.pipeline_handle)
            .unwrap();

        cmd_buffer.bind_pipeline(pipeline);

        if self.element_count > 0 {
            cmd_buffer.draw_indexed_indirect_count(
                indirect_arg_buffer.unwrap(),
                self.element_offset * INDIRECT_ARG_STRIDE as u64,
                count_buffer.unwrap(),
                self.element_offset,
                self.element_count,
                INDIRECT_ARG_STRIDE,
            );
        }

        for element in self.elements {
            element.draw(cmd_buffer);
        }
    }
}
