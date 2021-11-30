use crate::RendererHandle;

pub(crate) trait GpuSafeRotate {
    fn rotate(&mut self);
}

pub(crate) struct GpuSafePool<T: GpuSafeRotate> {
    num_cpu_frames: usize,
    cur_cpu_frame: usize,
    available: Vec<T>,
    in_use: Vec<Vec<T>>,
}

impl<T: GpuSafeRotate> GpuSafePool<T> {
    pub(crate) fn new(num_cpu_frames: usize) -> Self {
        Self {
            num_cpu_frames,
            cur_cpu_frame: 0,
            available: Vec::new(),
            in_use: (0..num_cpu_frames).map(|_| Vec::new()).collect(),
        }
    }

    pub(crate) fn rotate(&mut self) {
        let next_cpu_frame = (self.cur_cpu_frame + 1) % self.num_cpu_frames;
        self.available.append(&mut self.in_use[next_cpu_frame]);
        self.available.iter_mut().for_each(|x| x.rotate());
        self.cur_cpu_frame = next_cpu_frame;
    }

    pub(crate) fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> RendererHandle<T> {
        let result = if self.available.is_empty() {
            create_fn()
        } else {
            self.available.pop().unwrap()
        };
        RendererHandle::new(result)
    }

    pub(crate) fn release(&mut self, mut data: RendererHandle<T>) {
        self.in_use[self.cur_cpu_frame].push(data.peek());
    }
}