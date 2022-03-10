struct IndexBlock {
    indexes: Vec<u32>,
    base_index: u32,
}

impl Default for IndexBlock {
    fn default() -> Self {
        Self {
            indexes: Vec::new(),
            base_index: u32::MAX,
        }
    }
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

    pub fn acquire_index(&mut self) -> u32 {
        // todo: replace this vec of indices with some bit operations
        assert!(!self.indexes.is_empty());
        self.indexes.pop().unwrap()
    }

    pub fn release_index(&mut self, index: u32) {
        assert!(
            index >= self.base_index && index < self.base_index + self.indexes.capacity() as u32
        );
        self.indexes.push(index);
    }
}

pub struct IndexAllocator {
    block_size: u32,
    index_blocks: Vec<IndexBlock>,
}

impl IndexAllocator {
    pub fn new(block_size: u32) -> Self {
        Self {
            block_size,
            index_blocks: Vec::new(),
        }
    }

    pub fn acquire_index(&mut self) -> u32 {
        let block = self.acquire_index_block_mut();
        block.acquire_index()
    }

    pub fn release_index_ids(&mut self, indexes: &[u32]) {
        for index in indexes {
            let block_id = index / self.block_size;
            let block = &mut self.index_blocks[block_id as usize];
            block.release_index(*index);
        }
    }

    fn acquire_index_block_mut(&mut self) -> &mut IndexBlock {
        let mut most_free = 0;
        let mut most_free_idx = self.index_blocks.len();

        for i in 0..self.index_blocks.len() {
            let block = &self.index_blocks[i];
            let free_count = block.indexes.len();
            if free_count > most_free {
                most_free = free_count;
                most_free_idx = i;
            }
        }
        let index_blocks_len = self.index_blocks.len();
        if most_free_idx >= index_blocks_len {
            self.index_blocks.push(IndexBlock::new(
                index_blocks_len as u32 * self.block_size,
                self.block_size,
            ));
            most_free_idx = index_blocks_len;
        }
        &mut self.index_blocks[most_free_idx]
    }
}
