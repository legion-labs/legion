use lgn_graphics_api::Buffer;

use crate::{hl_gfx_api::HLCommandBuffer, resources::PipelineHandle, RenderContext};

use super::RenderElement;

#[derive(Clone)]
pub struct RenderStateSet {
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
    pub(crate) fn new(state_set: &RenderStateSet) -> Self {
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

    pub fn remove_cpu_element(&mut self, gpu_instance_id: u32) {
        for (index, matching) in self.elements.iter().enumerate() {
            if gpu_instance_id == matching.gpu_instance_id {
                self.elements.swap_remove(index);
                return;
            }
        }
    }

    pub fn _reset_cpu_elements(&mut self) {
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
        *aggregate_offset += u64::from(self.element_count);
    }

    pub fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
    ) {
        const INDIRECT_ARG_STRIDE: u64 = 20;

        let pipeline = render_context
            .pipeline_manager()
            .get_pipeline(self.state_set.pipeline_handle)
            .unwrap();

        cmd_buffer.bind_pipeline(pipeline);

        cmd_buffer.bind_descriptor_set(
            render_context.frame_descriptor_set().0,
            render_context.frame_descriptor_set().1,
        );
        cmd_buffer.bind_descriptor_set(
            render_context.view_descriptor_set().0,
            render_context.view_descriptor_set().1,
        );

        if self.element_count > 0 {
            cmd_buffer.draw_indexed_indirect_count(
                indirect_arg_buffer.unwrap(),
                self.element_offset * INDIRECT_ARG_STRIDE,
                count_buffer.unwrap(),
                self.element_offset,
                self.element_count,
                INDIRECT_ARG_STRIDE as u32,
            );
        }

        for element in &self.elements {
            element.draw(cmd_buffer);
        }
    }
}
