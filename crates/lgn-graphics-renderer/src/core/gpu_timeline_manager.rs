use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct GPUTimelineManager {
    inner: Arc<Mutex<RefCell<Inner>>>,
}

impl GPUTimelineManager {
    pub fn new(render_frame_capacity: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RefCell::new(Inner {
                render_frame_capacity,
                render_frame_index: 0,
                callbacks: (0..render_frame_capacity)
                    .map(|_x| CallbackBucket(Vec::new()))
                    .collect(),
            }))),
        }
    }

    #[allow(dead_code)]
    pub fn drop<T: Into<Box<dyn GPUTimelineCallback>>>(&self, dropper: T) {
        let guard = self.inner.lock().unwrap();
        let mut inner = guard.borrow_mut();
        let render_frame_index = inner.render_frame_index;
        inner.callbacks[render_frame_index as usize]
            .0
            .push(dropper.into());
    }

    pub fn flush(&self, frame_index: usize) {
        let guard = self.inner.lock().unwrap();
        let mut inner = guard.borrow_mut();

        // Move to the next frame. Now, we can safely free the memory. The GPU should
        // not have any implicit reference on some API objects/memory.
        {
            let next_render_frame = (frame_index as u64 + 1) % inner.render_frame_capacity;

            for callback in inner.callbacks[next_render_frame as usize].0.drain(..) {
                callback.execute();
            }

            inner.render_frame_index = next_render_frame;
        }
    }

    pub fn destroy(&self) {
        let guard = self.inner.lock().unwrap();
        let mut inner = guard.borrow_mut();

        for i in 0..inner.callbacks.len() {
            for callback in inner.callbacks[i].0.drain(..) {
                callback.execute();
            }
        }
    }
}

pub trait GPUTimelineCallback {
    fn execute(self: Box<Self>);
}

//
// Generic drop fn
//
pub struct GenericGPUTimelineCallback<T> {
    obj: T,
    drop_fn: fn(T),
}

impl<T> GPUTimelineCallback for GenericGPUTimelineCallback<T> {
    fn execute(self: Box<Self>) {
        let drop_fn = self.drop_fn;
        drop_fn(self.obj);
    }
}

struct CallbackBucket(pub Vec<Box<dyn GPUTimelineCallback>>);

struct Inner {
    pub render_frame_capacity: u64,
    pub render_frame_index: u64,
    pub callbacks: Vec<CallbackBucket>,
}

#[allow(unsafe_code, clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Inner {}

#[allow(unsafe_code)]
unsafe impl Sync for Inner {}
