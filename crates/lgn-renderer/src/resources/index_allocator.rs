use std::sync::{Arc, Mutex};

pub(crate) struct IndexBlock {
    indexes: Vec<u32>,
    base_index: u32,
}

impl IndexBlock {
    pub fn new(base_index: u32, block_size: u32) -> Self {
        let mut indexes = Vec::with_capacity(block_size as usize);
        indexes.reserve(block_size as usize);
        for i in 0..block_size {
            indexes.push(base_index + i as u32);
        }

        Self {
            indexes,
            base_index,
        }
    }

    pub fn acquire_index(&mut self) -> Option<u32> {
        self.indexes.pop()
    }

    pub fn release_index(&mut self, index: u32) {
        assert!(
            index >= self.base_index && index < self.base_index + self.indexes.capacity() as u32
        );
        self.indexes.push(index);
    }

    pub fn base_index(&self) -> u32 {
        self.base_index
    }
}

pub(crate) struct IndexAllocatorInner {
    block_size: u32,
    index_blocks: Vec<Option<IndexBlock>>,
}

#[derive(Clone)]
pub(crate) struct IndexAllocator {
    inner: Arc<Mutex<IndexAllocatorInner>>,
}

impl IndexAllocator {
    pub fn new(block_size: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(IndexAllocatorInner {
                block_size,
                index_blocks: Vec::new(),
            })),
        }
    }

    pub fn acquire_index_block(&self) -> IndexBlock {
        let mut inner = self.inner.lock().unwrap();

        let mut most_free = 0;
        let mut most_free_idx = inner.index_blocks.len();

        for i in 0..inner.index_blocks.len() {
            if let Some(block) = &inner.index_blocks[i] {
                let free_count = block.indexes.len();

                if free_count > most_free {
                    most_free = free_count;
                    most_free_idx = i;
                }
            }
        }
        if most_free_idx < inner.index_blocks.len() {
            return inner.index_blocks[most_free_idx].take().unwrap();
        }

        let result = IndexBlock::new(
            inner.index_blocks.len() as u32 * inner.block_size,
            inner.block_size,
        );
        inner.index_blocks.push(None);
        result
    }

    pub fn release_index_block(&self, block: IndexBlock) {
        let inner = &mut *self.inner.lock().unwrap();

        let block_id = block.base_index() / inner.block_size;
        assert!(inner.index_blocks[block_id as usize].is_none());

        inner.index_blocks[block_id as usize] = Some(block);
    }

    pub fn release_index_ids(&self, indexes: &[u32]) {
        let inner = &mut *self.inner.lock().unwrap();

        for index in indexes {
            let block_id = index / inner.block_size;

            if let Some(block) = &mut inner.index_blocks[block_id as usize] {
                block.release_index(*index);
            } else {
                panic!();
            }
        }
    }

    pub fn acquire_index(&self, mut index_block: IndexBlock) -> (IndexBlock, u32) {
        let mut new_index = u32::MAX;
        while new_index == u32::MAX {
            if let Some(index) = index_block.acquire_index() {
                new_index = index;
            } else {
                self.release_index_block(index_block);
                index_block = self.acquire_index_block();
            }
        }
        (index_block, new_index)
    }
}
