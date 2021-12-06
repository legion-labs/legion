use lgn_graphics_api::Range;

pub(crate) struct RangeAllocator {
    free_list: Vec<Range>,
    available: u64,
}

impl RangeAllocator {
    pub fn new(size: u64) -> Self {
        Self {
            free_list: vec![Range::new(0, size)],
            available: size,
        }
    }

    pub fn allocate(&mut self, size: u64) -> Option<Range> {
        let mut result = None;
        if !self.free_list.is_empty() {
            let mut remove_index = self.free_list.len();
            for index in 0..self.free_list.len() {
                let mut range = &mut self.free_list[index];
                let range_size = range.last - range.first;
                if range_size >= size {
                    self.available -= size;
                    result = Some(Range::new(range.first, range.first + size));
                    if range_size != size {
                        range.first += size;
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
        result
    }

    pub fn free(&mut self, free_range: Range) {
        assert!(free_range.first < free_range.last);

        let mut insert_index = self.free_list.len();
        for index in 0..self.free_list.len() {
            let mut next_range = &mut self.free_list[index];

            // Sanity check for overlapped bounds
            assert!(free_range.first < next_range.first || free_range.first >= next_range.last);
            assert!(free_range.last <= next_range.first || free_range.last > next_range.last);

            if free_range.last == next_range.first {
                next_range.first = free_range.first;
                break;
            } else if free_range.first == next_range.last {
                next_range.last = free_range.last;
                break;
            } else if free_range.last < next_range.first {
                insert_index = index;
                break;
            }
        }
        self.available += free_range.last - free_range.first;
        self.free_list.insert(insert_index, free_range);
    }
}
