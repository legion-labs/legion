use crate::Handle;

pub(crate) struct ObjectPool<T> {
    availables: Vec<T>,
    acquired_count: u32,
}

impl<T> ObjectPool<T> {
    pub(crate) fn new() -> Self {
        Self {
            availables: Vec::new(),
            acquired_count: 0,
        }
    }

    pub(crate) fn iter_mut(&'_ mut self) -> std::slice::IterMut<'_, T> {
        self.availables.iter_mut()
    }

    pub(crate) fn end_frame(&mut self) {
        assert_eq!(self.acquired_count, 0);
    }

    pub(crate) fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> Handle<T> {
        let result = if self.availables.is_empty() {
            create_fn()
        } else {
            self.availables.pop().unwrap()
        };
        self.acquired_count += 1;
        Handle::new(result)
    }

    pub(crate) fn release(&mut self, mut data: Handle<T>) {
        assert!(self.acquired_count > 0);
        self.availables.push(data.take());
        self.acquired_count -= 1;
    }
}
