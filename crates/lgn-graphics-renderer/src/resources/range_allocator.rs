use lgn_tracing::error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    begin: u64,
    end: u64,
}

#[allow(dead_code)]
impl Range {
    pub fn from_begin_end(begin: u64, end: u64) -> Self {
        assert!(begin <= end);
        Self { begin, end }
    }

    pub fn from_begin_size(begin: u64, size: u64) -> Self {
        Self::from_begin_end(begin, begin + size)
    }

    pub fn begin(&self) -> u64 {
        self.begin
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn size(&self) -> u64 {
        self.end - self.begin
    }
}

pub struct RangeAllocator {
    free_list: Vec<Range>,
    capacity: u64,
    available: u64,
    alloc_count: u32,
}

impl RangeAllocator {
    pub fn new(size: u64) -> Self {
        Self {
            free_list: vec![Range::from_begin_size(0, size)],
            capacity: size,
            available: size,
            alloc_count: 0,
        }
    }

    pub fn allocate(&mut self, size: u64) -> Option<Range> {
        let mut result = None;
        if !self.free_list.is_empty() {
            let mut remove_index = self.free_list.len();
            for index in 0..self.free_list.len() {
                let mut range = &mut self.free_list[index];
                let range_size = range.end - range.begin;
                if range_size >= size {
                    self.available -= size;
                    result = Some(Range::from_begin_size(range.begin, size));
                    if range_size != size {
                        range.begin += size;
                    } else {
                        remove_index = index;
                    }
                    break;
                }
            }

            if remove_index < self.free_list.len() {
                self.free_list.remove(remove_index);
            }
        }
        if result.is_some() {
            self.alloc_count += 1;
        }
        result
    }

    pub fn free(&mut self, free_range: Range) {
        assert!(self.alloc_count > 0 && free_range.begin < free_range.end);

        let mut insert_index = self.free_list.len();
        for index in 0..self.free_list.len() {
            let mut next_range = &mut self.free_list[index];

            // Sanity check for overlapped bounds
            assert!(free_range.end <= next_range.begin || free_range.begin >= next_range.end);

            if free_range.end == next_range.begin {
                next_range.begin = free_range.begin;
                break;
            } else if free_range.begin == next_range.end {
                next_range.end = free_range.end;
                break;
            } else if free_range.end < next_range.begin {
                insert_index = index;
                break;
            }
        }
        self.available += free_range.end - free_range.begin;
        assert!(self.available <= self.capacity);

        if self.free_list.is_empty() {
            self.free_list.push(free_range);
        } else if insert_index != self.free_list.len() {
            self.free_list.insert(insert_index, free_range);
        }

        self.alloc_count -= 1;
    }
}

impl Drop for RangeAllocator {
    fn drop(&mut self) {
        if self.alloc_count != 0 {
            error!("Leaking {} allocations.", self.alloc_count);
        }
    }
}
