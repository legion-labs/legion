use crate::{
    Buffer, BufferView, BufferViewDef, IndexBufferBinding, IndexType, VertexBufferBinding,
};

#[derive(Clone)]
pub struct BufferSubAllocation<AllocType> {
    pub buffer: Buffer,
    pub memory: AllocType,
    pub byte_offset: u64,
    pub size: u64,
}

impl<AllocType> BufferSubAllocation<AllocType> {
    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn vertex_buffer_binding(&self) -> VertexBufferBinding<'_> {
        VertexBufferBinding {
            buffer: &self.buffer,
            byte_offset: self.byte_offset,
        }
    }

    pub fn index_buffer_binding(&self, index_type: IndexType) -> IndexBufferBinding<'_> {
        IndexBufferBinding {
            buffer: &self.buffer,
            byte_offset: self.byte_offset,
            index_type,
        }
    }

    pub fn create_const_buffer_view(&self) -> BufferView {
        let buffer_view_def =
            BufferViewDef::as_const_buffer_with_offset(self.size, self.byte_offset);

        BufferView::from_buffer(&self.buffer, &buffer_view_def)
    }

    pub fn create_byte_address_buffer_view(&self, read_only: bool) -> BufferView {
        let buffer_view_def =
            BufferViewDef::as_byte_address_buffer(self.buffer.definition(), read_only);
        BufferView::from_buffer(&self.buffer, &buffer_view_def)
    }

    pub fn create_structured_buffer_view(&self, struct_size: u64, read_only: bool) -> BufferView {
        let buffer_view_def = BufferViewDef::as_structured_buffer_with_offset(
            self.size,
            struct_size,
            read_only,
            self.byte_offset,
        );
        BufferView::from_buffer(&self.buffer, &buffer_view_def)
    }
}
