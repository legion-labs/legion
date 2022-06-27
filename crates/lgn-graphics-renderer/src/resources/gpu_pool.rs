use lgn_core::Handle;

// use super::OnFrameEventHandler;

pub(crate) struct GpuSafePool<T> {
    num_cpu_frames: u64,
    cur_cpu_frame: u64,
    available: Vec<T>,
    in_use: Vec<Vec<T>>,
    acquired_count: u32,
}

impl<T> GpuSafePool<T> {
    pub(crate) fn new(num_cpu_frames: u64) -> Self {
        Self {
            num_cpu_frames,
            cur_cpu_frame: 0,
            available: Vec::new(),
            in_use: (0..num_cpu_frames).map(|_| Vec::new()).collect(),
            acquired_count: 0,
        }
    }

    pub(crate) fn begin_frame(&mut self, func: impl Fn(&mut T)) {
        let next_cpu_frame = (self.cur_cpu_frame + 1) % self.num_cpu_frames as u64;
        self.available
            .append(&mut self.in_use[next_cpu_frame as usize]);
        self.available.iter_mut().for_each(func);
        self.cur_cpu_frame = next_cpu_frame;
    }

    pub(crate) fn end_frame(&mut self, func: impl Fn(&mut T)) {
        assert_eq!(self.acquired_count, 0);
        self.in_use[self.cur_cpu_frame as usize]
            .iter_mut()
            .for_each(func);
    }

    pub(crate) fn acquire_or_create(&mut self, create_fn: impl FnOnce() -> T) -> Handle<T> {
        let result = if self.available.is_empty() {
            create_fn()
        } else {
            self.available.pop().unwrap()
        };
        self.acquired_count += 1;
        Handle::new(result)
    }

    pub(crate) fn release(&mut self, mut data: Handle<T>) {
        assert!(self.acquired_count > 0);
        self.in_use[self.cur_cpu_frame as usize].push(data.take());
        self.acquired_count -= 1;
    }
}
