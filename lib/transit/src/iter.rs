pub trait IterableQueue {
    type Item;
    fn iter(&self) -> QueueIterator<'_, Self, Self::Item>
    where
        Self: Sized;
    fn len_bytes(&self) -> usize;
    fn read_value_at_offset(&self, offset: usize) -> (Self::Item, usize);
}

pub struct QueueIterator<'a, QueueT, ValueT> {
    queue: &'a QueueT,
    offset: usize,
    phantom: std::marker::PhantomData<&'a ValueT>,
}

impl<'a, QueueT, ValueT> QueueIterator<'a, QueueT, ValueT> {
    pub fn begin(queue: &'a QueueT) -> Self {
        let offset = 0;
        Self {
            queue,
            offset,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<QueueT: IterableQueue, ValueT> core::iter::Iterator for QueueIterator<'_, QueueT, ValueT> {
    type Item = QueueT::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.queue.len_bytes() {
            None
        } else {
            let (obj, next_offset) = self.queue.read_value_at_offset(self.offset);
            self.offset = next_offset;
            Some(obj)
        }
    }
}
