use crate::{BufferDef, ResourceUsage};

use super::{deferred_drop::Drc, Buffer, Descriptor, GPUViewType, ShaderResourceType};

bitflags::bitflags! {
    pub struct BufferViewFlags: u8 {
        const RAW_BUFFER = 0x01;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BufferViewDef {
    pub gpu_view_type: GPUViewType,
    pub byte_offset: u64,
    pub element_count: u64,
    pub element_size: u64,
    pub buffer_view_flags: BufferViewFlags,
}

impl BufferViewDef {
    pub fn as_const_buffer_typed<T: Sized>() -> Self {
        Self::as_const_buffer_with_offset_typed::<T>(0)
    }

    pub fn as_const_buffer(element_size: u64) -> Self {
        Self::as_const_buffer_with_offset(element_size, 0)
    }

    pub fn as_const_buffer_with_offset_typed<T: Sized>(byte_offset: u64) -> Self {
        Self::as_const_buffer_with_offset(std::mem::size_of::<T>() as u64, byte_offset)
    }

    pub fn as_const_buffer_with_offset(element_size: u64, byte_offset: u64) -> Self {
        Self {
            gpu_view_type: GPUViewType::ConstantBuffer,
            byte_offset,
            element_count: 1,
            element_size: element_size as u64,
            buffer_view_flags: BufferViewFlags::empty(),
        }
    }

    pub fn as_structured_buffer_typed<T: Sized>(element_count: u64, read_only: bool) -> Self {
        Self::as_structured_buffer_with_offset_typed::<T>(element_count, read_only, 0)
    }

    pub fn as_structured_buffer(element_count: u64, element_size: u64, read_only: bool) -> Self {
        Self::as_structured_buffer_with_offset(element_count, element_size, read_only, 0)
    }

    pub fn as_structured_buffer_with_offset_typed<T: Sized>(
        element_count: u64,
        read_only: bool,
        byte_offset: u64,
    ) -> Self {
        Self::as_structured_buffer_with_offset(
            element_count,
            std::mem::size_of::<T>() as u64,
            read_only,
            byte_offset,
        )
    }

    pub fn as_structured_buffer_with_offset(
        element_count: u64,
        element_size: u64,
        read_only: bool,
        byte_offset: u64,
    ) -> Self {
        Self {
            gpu_view_type: if read_only {
                GPUViewType::ShaderResource
            } else {
                GPUViewType::UnorderedAccess
            },
            byte_offset,
            element_count,
            element_size,
            buffer_view_flags: BufferViewFlags::empty(),
        }
    }

    pub fn as_byte_address_buffer(element_count: u64, read_only: bool) -> Self {
        Self::as_byte_address_buffer_with_offset(element_count, read_only, 0)
    }

    pub fn as_byte_address_buffer_with_offset(
        element_count: u64,
        read_only: bool,
        byte_offset: u64,
    ) -> Self {
        Self {
            gpu_view_type: if read_only {
                GPUViewType::ShaderResource
            } else {
                GPUViewType::UnorderedAccess
            },
            byte_offset,
            element_count,
            element_size: 4,
            buffer_view_flags: BufferViewFlags::RAW_BUFFER,
        }
    }

    pub fn verify(&self, buffer_def: &BufferDef) {
        match self.gpu_view_type {
            GPUViewType::ConstantBuffer => {
                assert!(buffer_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_CONST_BUFFER));
                assert!(self.element_size > 0);
                assert!(self.element_count == 1);
                assert!(self.buffer_view_flags.is_empty());
            }
            GPUViewType::ShaderResource | GPUViewType::UnorderedAccess => {
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
                assert!(self.element_count >= 1);
            }
            GPUViewType::RenderTarget | GPUViewType::DepthStencil => {
                panic!();
            }
        }

        let upper_bound = self.byte_offset + self.element_count * self.element_size;
        assert!(upper_bound <= buffer_def.size);
    }
}

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
    pub fn from_buffer(buffer: &Buffer, definition: BufferViewDef) -> Self {
        definition.verify(buffer.definition());

        let device_context = buffer.device_context();
        let offset = definition.byte_offset;
        let size = definition.element_size * definition.element_count;

        Self {
            inner: device_context.deferred_dropper().new_drc(BufferViewInner {
                definition,
                buffer: buffer.clone(),
                offset,
                size,
            }),
        }
    }

    pub fn definition(&self) -> &BufferViewDef {
        &self.inner.definition
    }

    pub fn buffer(&self) -> &Buffer {
        &self.inner.buffer
    }

    pub fn offset(&self) -> u64 {
        self.inner.offset
    }

    pub fn size(&self) -> u64 {
        self.inner.size
    }

    pub fn is_compatible_with_descriptor(&self, descriptor: &Descriptor) -> bool {
        match descriptor.shader_resource_type {
            ShaderResourceType::ConstantBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::ConstantBuffer
            }
            ShaderResourceType::StructuredBuffer | ShaderResourceType::ByteAddressBuffer => {
                self.inner.definition.gpu_view_type == GPUViewType::ShaderResource
            }
            ShaderResourceType::RWStructuredBuffer | ShaderResourceType::RWByteAddressBuffer => {
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

#[derive(Clone, Copy)]
pub struct TransientBufferView {
    definition: BufferViewDef,
    buffer: *const Buffer,
    offset: u64,
    size: u64,
}

impl<'a> TransientBufferView {
    pub fn from_buffer(buffer: &Buffer, definition: BufferViewDef) -> Self {
        definition.verify(buffer.definition());

        let offset = definition.byte_offset;
        let size = definition.element_size * definition.element_count;

        Self {
            definition,
            buffer,
            offset,
            size,
        }
    }

    pub fn definition(&self) -> &BufferViewDef {
        &self.definition
    }

    #[allow(unsafe_code)]
    pub fn buffer(&self) -> &Buffer {
        unsafe { self.buffer.as_ref().unwrap() }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn is_compatible_with_descriptor(&self, descriptor: &Descriptor) -> bool {
        match descriptor.shader_resource_type {
            ShaderResourceType::ConstantBuffer => {
                self.definition.gpu_view_type == GPUViewType::ConstantBuffer
            }
            ShaderResourceType::StructuredBuffer | ShaderResourceType::ByteAddressBuffer => {
                self.definition.gpu_view_type == GPUViewType::ShaderResource
            }
            ShaderResourceType::RWStructuredBuffer | ShaderResourceType::RWByteAddressBuffer => {
                self.definition.gpu_view_type == GPUViewType::UnorderedAccess
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
