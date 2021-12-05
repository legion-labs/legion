use crate::RenderHandle;

use super::OnFrameEventHandler;

pub(crate) struct CpuPool<T: OnFrameEventHandler> {
    availables: Vec<T>,
    acquired_count: u32,
}

impl<T: OnFrameEventHandler> CpuPool<T> {
    pub(crate) fn new() -> Self {
        Self {
            availables: Vec::new(),
            acquired_count: 0,
        }
    }

    pub(crate) fn begin_frame(&mut self) {
        self.availables.iter_mut().for_each(T::on_begin_frame);
    }

    pub(crate) fn end_frame(&mut self) {
        assert_eq!(self.acquired_count, 0);
        self.availables.iter_mut().for_each(T::on_end_frame);
    }

    pub(crate) fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> RenderHandle<T> {
        let result = if self.availables.is_empty() {
            create_fn()
        } else {
            self.availables.pop().unwrap()
        };
        self.acquired_count += 1;
        RenderHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut data: RenderHandle<T>) {
        assert!(self.acquired_count > 0);
        self.availables.push(data.take());
        self.acquired_count -= 1;
    }
}
