use crate::Handle;

/// Pool of objects
pub struct ObjectPool<T> {
    availables: Vec<T>,
    acquired_count: u32,
}

impl<T> ObjectPool<T> {
    /// Construct an empty [`ObjectPool`]
    pub fn new() -> Self {
        Self {
            availables: Vec::new(),
            acquired_count: 0,
        }
    }

    /// Return the number of acquired objects
    pub fn acquired_count(&self) -> u32 {
        self.acquired_count
    }

    /// Return an iterator on available objects
    pub fn iter(&'_ self) -> std::slice::Iter<'_, T> {
        self.availables.iter()
    }

    /// Return a mut iterator on available objects
    pub fn iter_mut(&'_ mut self) -> std::slice::IterMut<'_, T> {
        self.availables.iter_mut()
    }

    /// Acquire or create an object in the pool. Returns a handle.
    pub fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> Handle<T> {
        let result = if self.availables.is_empty() {
            create_fn()
        } else {
            self.availables.pop().unwrap()
        };
        self.acquired_count += 1;
        Handle::new(result)
    }

    /// Release a handle.
    pub fn release(&mut self, mut data: Handle<T>) {
        assert!(self.acquired_count > 0);
        self.availables.push(data.take());
        self.acquired_count -= 1;
    }
}

impl<T: Default> Default for ObjectPool<T> {
    fn default() -> Self {
        Self {
            availables: Vec::default(),
            acquired_count: 0,
        }
    }
}
