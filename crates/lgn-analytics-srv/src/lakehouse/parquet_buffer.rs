use parquet::file::writer::TryClone;
use std::{io::Cursor, sync::Arc};

#[derive(Clone)]
pub struct InMemStream {
    _cursor: Arc<Cursor<Vec<u8>>>,
    cursor_ptr: *mut Cursor<Vec<u8>>, // until we can use get_mut_unchecked
}

#[allow(unsafe_code)]
unsafe impl Send for InMemStream {}

impl InMemStream {
    pub fn new(cursor: Arc<Cursor<Vec<u8>>>) -> Self {
        let cursor_ptr = Arc::as_ptr(&cursor) as *mut std::io::Cursor<Vec<u8>>;
        Self {
            _cursor: cursor,
            cursor_ptr,
        }
    }
}

impl std::io::Write for InMemStream {
    #[allow(unsafe_code)]
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        unsafe { (&mut *self.cursor_ptr).write(buf) }
    }

    #[allow(unsafe_code)]
    fn flush(&mut self) -> Result<(), std::io::Error> {
        unsafe { (&mut *self.cursor_ptr).flush() }
    }
}

impl std::io::Seek for InMemStream {
    #[allow(unsafe_code)]
    fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64, std::io::Error> {
        unsafe { (&mut *self.cursor_ptr).seek(pos) }
    }
}

impl TryClone for InMemStream {
    fn try_clone(&self) -> Result<Self, std::io::Error> {
        Ok(self.clone())
    }
}
