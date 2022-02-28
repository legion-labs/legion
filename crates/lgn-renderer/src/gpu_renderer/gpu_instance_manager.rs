use lgn_ecs::prelude::Entity;
use lgn_graphics_api::{BufferView, VertexBufferBinding};

use crate::{
    cgen,
    resources::{
        GpuDataManager, GpuVaTableForGpuInstance, IndexBlock, UnifiedStaticBuffer,
        UniformGPUDataUpdater,
    },
};

pub(crate) type GpuVaTableManager = GpuDataManager<Entity, cgen::cgen_type::GpuInstanceVATable>;

pub(crate) struct GpuInstanceVas {
    pub submesh_va: u32,
    pub material_va: u32,

    pub color_va: u32,
    pub transform_va: u32,
    pub picking_data_va: u32,
}

pub(crate) struct GpuInstanceManager {
    va_table_manager: GpuVaTableManager,
    va_table_adresses: GpuVaTableForGpuInstance,
}

impl GpuInstanceManager {
    pub fn new(static_buffer: &UnifiedStaticBuffer) -> Self {
        Self {
            va_table_manager: GpuVaTableManager::new(static_buffer, 64 * 1024, 4096),
            va_table_adresses: GpuVaTableForGpuInstance::new(static_buffer),
        }
    }

    pub fn add_gpu_instance(
        &mut self,
        entity: Entity,
        index_block: &mut Option<IndexBlock>,
        updater: &mut UniformGPUDataUpdater,
        instance_vas: &GpuInstanceVas,
    ) -> u32 {
        let (gpu_instance_id, va_table_address) =
            self.va_table_manager.alloc_gpu_data(entity, index_block);

        self.va_table_adresses
            .set_va_table_address_for_gpu_instance(
                updater,
                gpu_instance_id,
                va_table_address as u32,
            );

        let mut gpu_instance_va_table = cgen::cgen_type::GpuInstanceVATable::default();
        gpu_instance_va_table.set_mesh_description_va(instance_vas.submesh_va.into());
        gpu_instance_va_table.set_world_transform_va(instance_vas.transform_va.into());

        // Fallback to default material if we do not have a specific material set
        if instance_vas.material_va == u32::MAX {
            // gpu_instance_va_table
            //     .set_material_data_va(uniform_data.default_material_gpu_offset.into());
        } else {
            gpu_instance_va_table.set_material_data_va(instance_vas.material_va.into());
        }
        gpu_instance_va_table.set_instance_color_va(instance_vas.color_va.into());
        gpu_instance_va_table.set_picking_data_va(instance_vas.picking_data_va.into());

        updater.add_update_jobs(&[gpu_instance_va_table], va_table_address);

        gpu_instance_id
    }

    pub fn remove_gpu_instance(&mut self, entity: Entity) -> Option<Vec<u32>> {
        self.va_table_manager.remove_gpu_data(&entity)
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        self.va_table_adresses.vertex_buffer_binding()
    }

    pub fn structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        self.va_table_adresses
            .structured_buffer_view(struct_size, read_only)
    }

    pub fn return_index_block(&self, index_block: Option<IndexBlock>) {
        self.va_table_manager.return_index_block(index_block);
    }
}
