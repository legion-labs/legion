use crate::RenderHandle;

use super::OnNewFrame;

pub(crate) struct GpuSafePool<T: OnNewFrame> {
    num_cpu_frames: usize,
    cur_cpu_frame: usize,
    available: Vec<T>,
    in_use: Vec<Vec<T>>,
}

impl<T: OnNewFrame> GpuSafePool<T> {
    pub(crate) fn new(num_cpu_frames: usize) -> Self {
        Self {
            num_cpu_frames,
            cur_cpu_frame: 0,
            available: Vec::new(),
            in_use: (0..num_cpu_frames).map(|_| Vec::new()).collect(),
        }
    }

    pub(crate) fn new_frame(&mut self) {
        let next_cpu_frame = (self.cur_cpu_frame + 1) % self.num_cpu_frames;
        self.available.append(&mut self.in_use[next_cpu_frame]);
        self.available.iter_mut().for_each(|x| x.on_new_frame());
        self.cur_cpu_frame = next_cpu_frame;
    }

    pub(crate) fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> RenderHandle<T> {
        let result = if self.available.is_empty() {
            create_fn()
        } else {
            self.available.pop().unwrap()
        };
        RenderHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut data: RenderHandle<T>) {
        self.in_use[self.cur_cpu_frame].push(data.take());
    }
}
