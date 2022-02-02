use crate::BufferViewDef;

use super::{deferred_drop::Drc, Buffer, Descriptor, GPUViewType, ShaderResourceType};

struct BufferViewInner {
    definition: BufferViewDef,
    buffer: Buffer,
    offset: u64,
    size: u64,
}

#[derive(Clone)]
pub struct BufferView {
    inner: Drc<BufferViewInner>,
}

impl BufferView {
    pub fn from_buffer(buffer: &Buffer, view_def: &BufferViewDef) -> Self {
        view_def.verify(buffer.definition());

        let device_context = buffer.device_context();
        let offset = view_def.byte_offset;
        let size = view_def.element_size * view_def.element_count;

        Self {
            inner: device_context.deferred_dropper().new_drc(BufferViewInner {
                definition: *view_def,
                buffer: buffer.clone(),
                offset,
                size,
            }),
        }
    }

    pub fn definition(&self) -> &BufferViewDef {
        &self.inner.definition
    }

    pub(crate) fn buffer(&self) -> &Buffer {
        &self.inner.buffer
    }

    pub(crate) fn offset(&self) -> u64 {
        self.inner.offset
    }

    pub(crate) fn size(&self) -> u64 {
        self.inner.size
    }

    pub(crate) fn is_compatible_with_descriptor(&self, descriptor: &Descriptor) -> bool {
        match descriptor.shader_resource_type {
            ShaderResourceType::ConstantBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::ConstantBuffer
            }
            ShaderResourceType::StructuredBuffer | ShaderResourceType::ByteAdressBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::ShaderResource
            }
            ShaderResourceType::RWStructuredBuffer | ShaderResourceType::RWByteAdressBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::UnorderedAccess
            }
            ShaderResourceType::Sampler
            | ShaderResourceType::Texture2D
            | ShaderResourceType::RWTexture2D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::Texture3D
            | ShaderResourceType::RWTexture3D
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => false,
        }
    }
}
