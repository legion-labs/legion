use lgn_app::{App, Plugin};
use lgn_graphics_api::{PagedBufferAllocation, VertexBufferBinding};

use crate::cgen;

use super::{IndexAllocator, UnifiedStaticBuffer, UniformGPUData, UniformGPUDataUpdater};

pub struct GpuDataPlugin {
    static_buffer: UnifiedStaticBuffer,
}

impl GpuDataPlugin {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            static_buffer: static_buffer.clone(),
        }
    }
}

pub(crate) type GpuInstanceIdAllocator = IndexAllocator;

pub(crate) type GpuInstanceTransform = UniformGPUData<cgen::cgen_type::GpuInstanceTransform>;
pub(crate) type GpuInstanceVATable = UniformGPUData<cgen::cgen_type::GpuInstanceVATable>;
pub(crate) type GpuInstanceColor = UniformGPUData<cgen::cgen_type::GpuInstanceColor>;
pub(crate) type GpuInstancePickingData = UniformGPUData<cgen::cgen_type::GpuInstancePickingData>;

impl Plugin for GpuDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GpuInstanceIdAllocator::new(4096));

        app.insert_resource(GpuInstanceTransform::new(&self.static_buffer, 64 * 1024));
        app.insert_resource(GpuInstanceVATable::new(&self.static_buffer, 64 * 1024));
        app.insert_resource(GpuInstanceColor::new(&self.static_buffer, 64 * 1024));
        app.insert_resource(GpuInstancePickingData::new(&self.static_buffer, 64 * 1024));

        app.insert_resource(GpuVaTableForGpuInstance::new(&self.static_buffer));
    }
}

pub(crate) struct GpuVaTableForGpuInstance {
    static_allocation: PagedBufferAllocation,
}

impl GpuVaTableForGpuInstance {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            static_allocation: static_buffer.allocate_segment(4 * 1024 * 1024),
        }
    }

    pub fn set_va_table_address_for_gpu_instance(
        &self,
        updater: &mut UniformGPUDataUpdater,
        gpu_instance: u32,
        va_table_address: u32,
    ) {
        let offset_for_gpu_instance = self.static_allocation.offset() + u64::from(gpu_instance) * 4;

        updater.add_update_jobs(&[va_table_address], offset_for_gpu_instance);
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.static_allocation.vertex_buffer_binding()
    }
}
