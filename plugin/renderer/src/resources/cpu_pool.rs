use crate::RenderHandle;

use super::OnNewFrame;

pub(crate) struct CpuPool<T: OnNewFrame> {
    availables: Vec<T>,
}

impl<T: OnNewFrame> CpuPool<T> {
    pub(crate) fn new() -> Self {
        Self {
            availables: Vec::new(),
        }
    }

    pub(crate) fn new_frame(&mut self) {
        self.availables.iter_mut().for_each(|x| x.on_new_frame());
    }

    pub(crate) fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> RenderHandle<T> {
        let result = if self.availables.is_empty() {
            create_fn()
        } else {
            self.availables.pop().unwrap()
        };
        RenderHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut data: RenderHandle<T>) {
        self.availables.push(data.take());
    }
}
