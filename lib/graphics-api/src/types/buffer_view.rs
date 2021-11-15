use crate::GfxResult;

use super::{deferred_drop::Drc, Buffer, BufferDef, BufferViewFlags, GPUViewType, ResourceUsage};
#[cfg(any(feature = "vulkan"))]
use super::{Descriptor, ShaderResourceType};

#[derive(Clone, Copy, Debug)]
pub struct BufferViewDef {
    pub gpu_view_type: GPUViewType,
    pub byte_offset: u64,
    pub element_count: u64,
    pub element_size: u64,
    pub buffer_view_flags: BufferViewFlags,
}

// const buffer : offset, size
// structbuffer

impl BufferViewDef {
    pub fn as_const_buffer(buffer_def: &BufferDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::ConstantBufferView,
            byte_offset: 0,
            element_count: 1,
            element_size: buffer_def.size,
            buffer_view_flags: BufferViewFlags::empty(),
        }
    }

    pub fn as_structured_buffer(buffer_def: &BufferDef, struct_size: u64, read_only: bool) -> Self {
        assert!(buffer_def.size % struct_size == 0);
        Self {
            gpu_view_type: if read_only {
                GPUViewType::ShaderResourceView
            } else {
                GPUViewType::UnorderedAccessView
            },
            byte_offset: 0,
            element_count: buffer_def.size / struct_size,
            element_size: struct_size,
            buffer_view_flags: BufferViewFlags::empty(),
        }
    }

    pub fn as_byte_address_buffer(buffer_def: &BufferDef, read_only: bool) -> Self {
        assert!(buffer_def.size % 4 == 0);
        Self {
            gpu_view_type: if read_only {
                GPUViewType::ShaderResourceView
            } else {
                GPUViewType::UnorderedAccessView
            },
            byte_offset: 0,
            element_count: buffer_def.size / 4,
            element_size: 0,
            buffer_view_flags: BufferViewFlags::RAW_BUFFER,
        }
    }

    pub fn verify(&self, buffer_def: &BufferDef) {
        match self.gpu_view_type {
            GPUViewType::ConstantBufferView => {
                assert!(buffer_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_CONST_BUFFER));
                assert!(self.element_size > 0);
                assert!(self.byte_offset == 0);
                assert!(self.element_count == 1);
                assert!(self.buffer_view_flags.is_empty());
            }
            GPUViewType::ShaderResourceView | GPUViewType::UnorderedAccessView => {
                assert!(buffer_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_SHADER_RESOURCE));
                if self
                    .buffer_view_flags
                    .intersects(BufferViewFlags::RAW_BUFFER)
                {
                    assert!(self.element_size == 4);
                } else {
                    assert!(self.element_size > 0);
                };
                assert!(self.byte_offset % self.element_size == 0);
                assert!(self.element_count >= 1);
            }
            GPUViewType::RenderTargetView | GPUViewType::DepthStencilView => {
                panic!();
            }
        }

        let upper_bound = self.byte_offset + self.element_count * self.element_size;
        assert!(upper_bound <= buffer_def.size);
    }
}

#[derive(Clone)]
pub struct BufferViewInner {
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
    pub fn from_buffer(buffer: &Buffer, view_def: &BufferViewDef) -> GfxResult<Self> {
        view_def.verify(buffer.definition());

        let device_context = buffer.device_context();
        let offset = view_def.byte_offset;
        let size = view_def.element_size * view_def.element_count;

        Ok(Self {
            inner: device_context.deferred_dropper().new_drc(BufferViewInner {
                definition: *view_def,
                buffer: buffer.clone(),
                offset,
                size,
            }),
        })
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn buffer(&self) -> &Buffer {
        &self.inner.buffer
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn offset(&self) -> u64 {
        self.inner.offset
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn size(&self) -> u64 {
        self.inner.size
    }

    #[cfg(any(feature = "vulkan"))]
    pub(crate) fn is_compatible_with_descriptor(&self, descriptor: &Descriptor) -> bool {
        match descriptor.shader_resource_type {
            ShaderResourceType::ConstantBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::ConstantBufferView
            }
            ShaderResourceType::StructuredBuffer | ShaderResourceType::ByteAdressBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::ShaderResourceView
            }
            ShaderResourceType::RWStructuredBuffer | ShaderResourceType::RWByteAdressBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::UnorderedAccessView
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
