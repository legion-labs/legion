use lgn_graphics_api::{BufferView, VertexBufferBinding};

use crate::{
    cgen,
    core::{GpuUploadManager, RenderObjectId},
    gpu_renderer::{GpuInstanceId, GpuVaTableForGpuInstance},
    resources::{GpuDataManager, UnifiedStaticBuffer},
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

pub(crate) struct MeshInstanceManager {
    pub(crate) transform_manager: GpuEntityTransformManager,
    pub(crate) color_manager: GpuEntityColorManager,
    pub(crate) picking_data_manager: GpuPickingDataManager,
    pub(crate) va_table_manager: GpuVaTableManager,
    pub(crate) va_table_adresses: GpuVaTableForGpuInstance,
    // pub(crate) added_render_elements: Vec<RenderElement>,
    // pub(crate) removed_gpu_instance_ids: Vec<GpuInstanceId>,
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
