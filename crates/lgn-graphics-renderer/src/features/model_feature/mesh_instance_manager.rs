use lgn_graphics_api::{BufferView, BufferViewDef, ResourceUsage, VertexBufferBinding};

use crate::{
    cgen,
    core::{
        BinaryWriter, GpuUploadManager, RenderCommandBuilder, RenderObjectId, UploadGPUBuffer,
        UploadGPUResource,
    },
    resources::{
        GpuDataAllocation, GpuDataManager, StaticBufferAllocation, StaticBufferView,
        UnifiedStaticBuffer, UpdateUnifiedStaticBufferCommand,
    },
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct MeshInstanceKey {
    pub(crate) render_object_id: RenderObjectId,
    pub(crate) mesh_index: usize,
}

type GpuPickingDataManager =
    GpuDataManager<RenderObjectId, cgen::cgen_type::GpuInstancePickingData>;
type GpuEntityColorManager = GpuDataManager<RenderObjectId, cgen::cgen_type::GpuInstanceColor>;
type GpuEntityTransformManager = GpuDataManager<RenderObjectId, cgen::cgen_type::TransformData>;
type GpuVaTableManager = GpuDataManager<MeshInstanceKey, cgen::cgen_type::GpuInstanceVATable>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuInstanceId(pub u32);

impl GpuInstanceId {
    pub fn index(self) -> u32 {
        self.0
    }
}

pub(crate) struct GpuInstanceVas {
    pub submesh_va: u32,
    pub material_va: u32,
    pub color_va: u32,
    pub transform_va: u32,
    pub picking_data_va: u32,
}

pub struct GpuVaTableForGpuInstance {
    static_allocation: StaticBufferAllocation,
    static_buffer_view: StaticBufferView,
}

impl GpuVaTableForGpuInstance {
    pub(crate) fn new(gpu_heap: &UnifiedStaticBuffer) -> Self {
        let element_count = 1024 * 1024;
        let element_size = std::mem::size_of::<u32>() as u64;
        let static_allocation = gpu_heap.allocate(
            element_count * element_size,
            ResourceUsage::AS_SHADER_RESOURCE | ResourceUsage::AS_VERTEX_BUFFER,
        );

        let buffer_view = static_allocation.create_view(BufferViewDef::as_structured_buffer(
            element_count,
            element_size,
            true,
        ));

        Self {
            static_allocation,
            static_buffer_view: buffer_view,
        }
    }

    pub(crate) fn set_va_table_address_for_gpu_instance(
        &self,
        render_commands: &mut RenderCommandBuilder,
        gpu_data_allocation: GpuDataAllocation,
    ) {
        let offset_for_gpu_instance =
            self.static_allocation.byte_offset() + u64::from(gpu_data_allocation.index()) * 4;

        let va = u32::try_from(gpu_data_allocation.gpuheap_addr()).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(&va);

        render_commands.push(UpdateUnifiedStaticBufferCommand {
            src_buffer: binary_writer.take(),
            dst_offset: offset_for_gpu_instance,
        });
    }

    pub(crate) fn sync_set_va_table_address_for_gpu_instance(
        &self,
        gpu_upload: &GpuUploadManager,
        gpu_data_allocation: GpuDataAllocation,
    ) {
        let offset_for_gpu_instance =
            self.static_allocation.byte_offset() + u64::from(gpu_data_allocation.index()) * 4;

        let va = u32::try_from(gpu_data_allocation.gpuheap_addr()).unwrap();

        let mut binary_writer = BinaryWriter::new();
        binary_writer.write(&va);

        gpu_upload.push(UploadGPUResource::Buffer(UploadGPUBuffer {
            src_data: binary_writer.take(),
            dst_buffer: self.static_allocation.buffer().clone(),
            dst_offset: offset_for_gpu_instance,
        }));
    }

    pub(crate) fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        self.static_allocation.vertex_buffer_binding()
    }

    pub(crate) fn structured_buffer_view(&self) -> &BufferView {
        self.static_buffer_view.buffer_view()
    }
}

pub(crate) struct MeshInstanceManager {
    pub(crate) transform_manager: GpuEntityTransformManager,
    pub(crate) color_manager: GpuEntityColorManager,
    pub(crate) picking_data_manager: GpuPickingDataManager,
    pub(crate) va_table_manager: GpuVaTableManager,
    pub(crate) va_table_adresses: GpuVaTableForGpuInstance,
}

impl MeshInstanceManager {
    pub fn new(gpu_heap: &UnifiedStaticBuffer, gpu_upload_manager: &GpuUploadManager) -> Self {
        Self {
            // TODO(vdbdd): as soon as we have a stable ID, we can move the transforms in their own manager.
            transform_manager: GpuEntityTransformManager::new(gpu_heap, 1024, gpu_upload_manager),
            color_manager: GpuEntityColorManager::new(gpu_heap, 256, gpu_upload_manager),
            picking_data_manager: GpuPickingDataManager::new(gpu_heap, 1024, gpu_upload_manager),
            va_table_manager: GpuVaTableManager::new(gpu_heap, 4096, gpu_upload_manager),
            va_table_adresses: GpuVaTableForGpuInstance::new(gpu_heap),
            // added_render_elements: Vec::new(),
            // removed_gpu_instance_ids: Vec::new(),
        }
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding {
        self.va_table_adresses.vertex_buffer_binding()
    }

    pub fn structured_buffer_view(&self) -> &BufferView {
        self.va_table_adresses.structured_buffer_view()
    }
}
