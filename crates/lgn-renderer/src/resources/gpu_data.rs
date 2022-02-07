use lgn_app::{App, Plugin};
use lgn_ecs::prelude::Commands;
use lgn_graphics_api::{PagedBufferAllocation, VertexBufferBinding};

use crate::{cgen, components::MaterialComponent};

use super::{
    IndexAllocator, IndexBlock, UnifiedStaticBuffer, UniformGPUData, UniformGPUDataUpdater,
};

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

pub(crate) type GpuInstanceTransform = UniformGPUData<cgen::cgen_type::GpuInstanceTransform>;
pub(crate) type GpuInstanceVATable = UniformGPUData<cgen::cgen_type::GpuInstanceVATable>;
pub(crate) type GpuInstanceColor = UniformGPUData<cgen::cgen_type::GpuInstanceColor>;
pub(crate) type GpuInstancePickingData = UniformGPUData<cgen::cgen_type::GpuInstancePickingData>;
pub(crate) type GpuMaterialData = UniformGPUData<cgen::cgen_type::MaterialData>;

pub struct GpuUniformData {
    pub gpu_instance_id_allocator: IndexAllocator,
    pub gpu_texture_id_allocator: IndexAllocator,
    pub gpu_material_id_allocator: IndexAllocator,

    pub gpu_instance_transform: GpuInstanceTransform,
    pub gpu_instance_va_table: GpuInstanceVATable,
    pub gpu_instance_color: GpuInstanceColor,
    pub gpu_instance_picking_data: GpuInstancePickingData,
    pub gpu_material_data: GpuMaterialData,

    pub default_material_gpu_offset: u32,
}

impl GpuUniformData {
    fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            gpu_instance_id_allocator: IndexAllocator::new(4096),
            gpu_texture_id_allocator: IndexAllocator::new(256),
            gpu_material_id_allocator: IndexAllocator::new(256),
            gpu_instance_transform: GpuInstanceTransform::new(static_buffer, 64 * 1024),
            gpu_instance_va_table: GpuInstanceVATable::new(static_buffer, 64 * 1024),
            gpu_instance_color: GpuInstanceColor::new(static_buffer, 64 * 1024),
            gpu_instance_picking_data: GpuInstancePickingData::new(static_buffer, 64 * 1024),
            gpu_material_data: GpuMaterialData::new(static_buffer, 64 * 1024),
            default_material_gpu_offset: u32::MAX,
        }
    }

    pub fn initialize_default_material(&mut self, mut commands: Commands<'_, '_>) {
        let mut data_context = GpuUniformDataContext::new(self);
        let default = MaterialComponent::new(&mut data_context);
        std::mem::drop(data_context);

        self.default_material_gpu_offset = default.gpu_offset();
        commands.spawn().insert(default);
    }
}

pub struct GpuUniformDataContext<'a> {
    pub uniform_data: &'a GpuUniformData,

    pub gpu_instance_id_block: Option<IndexBlock>,
    pub gpu_texture_id_block: Option<IndexBlock>,
    pub gpu_material_id_block: Option<IndexBlock>,
}

impl<'a> Drop for GpuUniformDataContext<'a> {
    fn drop(&mut self) {
        if let Some(index_block) = self.gpu_instance_id_block.take() {
            self.uniform_data
                .gpu_instance_id_allocator
                .release_index_block(index_block);
        }
        if let Some(index_block) = self.gpu_texture_id_block.take() {
            self.uniform_data
                .gpu_texture_id_allocator
                .release_index_block(index_block);
        }
        if let Some(index_block) = self.gpu_material_id_block.take() {
            self.uniform_data
                .gpu_material_id_allocator
                .release_index_block(index_block);
        }
    }
}

impl<'a> GpuUniformDataContext<'a> {
    pub fn new(uniform_data: &'a GpuUniformData) -> Self {
        Self {
            uniform_data,
            gpu_instance_id_block: None,
            gpu_texture_id_block: None,
            gpu_material_id_block: None,
        }
    }

    pub fn aquire_gpu_instance_id(&mut self) -> u32 {
        self.uniform_data
            .gpu_instance_id_allocator
            .acquire_index(&mut self.gpu_instance_id_block)
    }

    pub fn aquire_gpu_texture_id(&mut self) -> u32 {
        self.uniform_data
            .gpu_texture_id_allocator
            .acquire_index(&mut self.gpu_texture_id_block)
    }

    pub fn aquire_gpu_material_id(&mut self) -> u32 {
        self.uniform_data
            .gpu_material_id_allocator
            .acquire_index(&mut self.gpu_material_id_block)
    }
}

impl Plugin for GpuDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GpuUniformData::new(&self.static_buffer));

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
