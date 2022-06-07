use lgn_graphics_api::DeviceContext;

use crate::{cgen::cgen_type::PickingData, resources::GpuBufferWithReadback};

// TODO(jsg): Move this somewhere else to be able to remove this struct entirely.
pub struct PickingRenderPass {
    pub(crate) count_buffer: GpuBufferWithReadback,
    pub(crate) picked_buffer: GpuBufferWithReadback,
}

impl PickingRenderPass {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            count_buffer: GpuBufferWithReadback::new(device_context, 4),
            picked_buffer: GpuBufferWithReadback::new(
                device_context,
                16 * 1024 * std::mem::size_of::<PickingData>() as u64,
            ),
        }
    }
}
