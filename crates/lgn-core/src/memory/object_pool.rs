use crate::Handle;

pub struct ObjectPool<T> {
    availables: Vec<T>,
    acquired_count: u32,
}

impl<T> ObjectPool<T> {
    pub fn new() -> Self {
        Self {
            availables: Vec::new(),
            acquired_count: 0,
        }
    }

    pub fn acquired_count(&self) -> u32 {
        self.acquired_count
    }

    pub fn availables(&'_ self) -> std::slice::Iter<'_, T> {
        self.availables.iter()
    }

    pub fn availables_mut(&'_ mut self) -> std::slice::IterMut<'_, T> {
        self.availables.iter_mut()
    }

    pub fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> Handle<T> {
        let result = if self.availables.is_empty() {
            create_fn()
        } else {
            self.availables.pop().unwrap()
        };
        self.acquired_count += 1;
        Handle::new(result)
    }

    pub fn release(&mut self, mut data: Handle<T>) {
        assert!(self.acquired_count > 0);
        self.availables.push(data.take());
        self.acquired_count -= 1;
    }
}
