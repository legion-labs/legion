use crate::{
    Buffer, BufferView, BufferViewDef, CommandBuffer, IndexBufferBinding, IndexType,
    VertexBufferBinding,
};

#[derive(Clone, Copy)]
pub struct Range {
    pub first: u64,
    pub last: u64,
}

impl Range {
    pub fn new(first: u64, last: u64) -> Self {
        Self { first, last }
    }
}

#[derive(Clone)]
pub struct BufferSubAllocation<AllocType> {
    pub buffer: Buffer,
    pub memory: AllocType,
    pub range: Range,
}

impl<AllocType> BufferSubAllocation<AllocType> {
    pub fn offset(&self) -> u64 {
        self.range.first
    }

    pub fn size(&self) -> u64 {
        self.range.last - self.range.first
    }

    pub fn bind_allocation_as_vertex_buffer(&self, cmd_buffer: &CommandBuffer) {
        cmd_buffer
            .cmd_bind_vertex_buffers(
                0,
                &[VertexBufferBinding {
                    buffer: &self.buffer,
                    byte_offset: self.range.first,
                }],
            )
            .unwrap();
    }

    pub fn bind_allocation_as_index_buffer(
        &self,
        cmd_buffer: &CommandBuffer,
        index_type: IndexType,
    ) {
        cmd_buffer
            .cmd_bind_index_buffer(&IndexBufferBinding {
                buffer: &self.buffer,
                byte_offset: self.range.first,
                index_type,
            })
            .unwrap();
    }

    pub fn const_buffer_view_for_allocation(&self) -> BufferView {
        let size = self.range.last - self.range.first;
        let buffer_view_def = BufferViewDef::as_const_buffer_with_offset(size, self.range.first);

        BufferView::from_buffer(&self.buffer, &buffer_view_def).unwrap()
    }

    pub fn byte_address_buffer_view_for_allocation(&self, read_only: bool) -> BufferView {
        let buffer_view_def =
            BufferViewDef::as_byte_address_buffer(self.buffer.definition(), read_only);
        BufferView::from_buffer(&self.buffer, &buffer_view_def).unwrap()
    }
}
