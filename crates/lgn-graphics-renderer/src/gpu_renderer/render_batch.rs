use lgn_graphics_api::Buffer;

use crate::{hl_gfx_api::HLCommandBuffer, resources::PipelineHandle, RenderContext};

use super::{GpuInstanceId, RenderElement};

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
    count_offset: u64,
}

impl RenderBatch {
    pub(crate) fn new(state_set: &RenderStateSet) -> Self {
        Self {
            state_set: state_set.clone(),
            elements: vec![],
            element_count: 0,
            element_offset: u64::MAX,
            count_offset: u64::MAX,
        }
    }

    pub fn add_cpu_element(&mut self, element: &RenderElement) {
        self.elements.push(*element);
    }

    pub fn remove_cpu_element(&mut self, gpu_instance_id: GpuInstanceId) {
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

    pub fn calculate_indirect_offsets(
        &mut self,
        count_buffer_offset: &mut u64,
        indirect_arg_buffer_offset: &mut u64,
    ) {
        self.count_offset = *count_buffer_offset;
        self.element_offset = *indirect_arg_buffer_offset;

        *count_buffer_offset += 1;
        *indirect_arg_buffer_offset += u64::from(self.element_count);
    }

    pub fn draw(
        &self,
        render_context: &RenderContext<'_>,
        cmd_buffer: &mut HLCommandBuffer<'_>,
        indirect_arg_buffer: Option<&Buffer>,
        count_buffer: Option<&Buffer>,
    ) {
        const COUNT_STRIDE: u64 = 4;
        const INDIRECT_ARG_STRIDE: u64 = 20;

        if self.element_count > 0 || !self.elements.is_empty() {
            let pipeline = render_context
                .pipeline_manager()
                .get_pipeline(self.state_set.pipeline_handle)
                .unwrap();

            cmd_buffer.bind_pipeline(pipeline);

            render_context.bind_default_descriptor_sets(cmd_buffer);

            if self.element_count > 0 {
                cmd_buffer.draw_indexed_indirect_count(
                    indirect_arg_buffer.unwrap(),
                    self.element_offset * INDIRECT_ARG_STRIDE,
                    count_buffer.unwrap(),
                    self.count_offset * COUNT_STRIDE,
                    self.element_count,
                    INDIRECT_ARG_STRIDE as u32,
                );
            }

            for element in &self.elements {
                element.draw(cmd_buffer);
            }
        }
    }
}
