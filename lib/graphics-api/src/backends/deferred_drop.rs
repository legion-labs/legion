#![allow(unsafe_code)]
use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;
use std::process::abort;
use std::ptr::NonNull;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    atomic::{self, Ordering},
    Arc, Mutex,
};

#[derive(Debug, Clone)]
pub struct DeferredDropper {
    inner: Arc<Mutex<RefCell<DeferredDropperInner>>>,
}

impl DeferredDropper {
    pub fn new(render_frame_capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            inner: Arc::new(Mutex::new(RefCell::new(DeferredDropperInner {
                render_frame_capacity,
                render_frame_index: 0,
                buckets: (0..render_frame_capacity)
                    .map(|_x| ObjectBucket(Vec::new()))
                    .collect(),
                sender: tx,
                receiver: rx,
            }))),
        }
    }

    pub fn new_drc<T>(&self, data: T) -> Drc<T> {
        let guard = self.inner.lock().unwrap();
        let inner = guard.borrow();
        Drc::new(inner.sender.clone(), data)
    }

    pub fn flush(&self) {
        let guard = self.inner.lock().unwrap();
        let mut inner = guard.borrow_mut();

        // Flush queue in the 'current frame' bucket.
        {
            let current_render_frame = inner.render_frame_index;
            while let Ok(object) = inner.receiver.try_recv() {
                inner.buckets[current_render_frame].0.push(object);
            }
        }
        // Move to the next frame. Now, we can safely free the memory. The GPU should not have any
        // implicit reference on some API objects/memory.
        {
            let next_render_frame = (inner.render_frame_index + 1) % inner.render_frame_capacity;

            inner.buckets[next_render_frame].0.drain(..);
            inner.render_frame_index = next_render_frame;
        }
    }

    pub fn destroy(&self) {
        let guard = self.inner.lock().unwrap();
        let mut inner = guard.borrow_mut();

        for i in 0..inner.buckets.len() {
            inner.buckets[i].0.drain(..);
        }

        while let Ok(object) = inner.receiver.try_recv() {
            drop(object);
        }
    }
}

impl Drop for DeferredDropper {
    fn drop(&mut self) {
        let guard = self.inner.lock().unwrap();
        let inner = guard.borrow();
        assert!(inner.receiver.try_recv().is_err());
        for i in 0..inner.buckets.len() {
            assert!(inner.buckets[i].0.is_empty());
        }
    }
}

#[derive(Debug)]
pub struct ObjectBucket(pub Vec<Box<dyn Any>>);

#[derive(Debug)]
struct DeferredDropperInner {
    pub render_frame_capacity: usize,
    pub render_frame_index: usize,
    pub buckets: Vec<ObjectBucket>,
    pub sender: Sender<Box<dyn Any>>,
    pub receiver: Receiver<Box<dyn Any>>,
}

unsafe impl Send for DeferredDropperInner {}

unsafe impl Sync for DeferredDropperInner {}

#[derive(Debug)]
struct DrcInner<T> {
    strong: atomic::AtomicUsize,
    tx: Sender<Box<dyn Any>>,
    data: T,
}

#[derive(Debug)]
pub struct Drc<T: 'static> {
    ptr: NonNull<DrcInner<T>>,
}

impl<T> Drc<T> {
    pub fn new(tx: Sender<Box<dyn Any>>, data: T) -> Self {
        let x = Box::new(DrcInner {
            strong: atomic::AtomicUsize::new(1),
            tx,
            data,
        });

        Self::from_inner(Box::leak(x).into())
    }

    fn from_inner(ptr: NonNull<DrcInner<T>>) -> Self {
        Self { ptr }
    }

    fn inner(&self) -> &DrcInner<T> {
        unsafe { self.ptr.as_ref() }
    }

    unsafe fn drop_slow(&mut self) {
        let boxed = Box::<dyn Any>::from_raw(self.ptr.as_ptr());
        let tx = &self.ptr.as_ref().tx;
        tx.send(boxed).unwrap();
    }
}

unsafe impl<T> Send for Drc<T> where T: Send {}

unsafe impl<T> Sync for Drc<T> where T: Sync {}

impl<T> Clone for Drc<T> {
    fn clone(&self) -> Self {
        let old_size = self.inner().strong.fetch_add(1, Ordering::Relaxed);

        const MAX_REFCOUNT: usize = (isize::MAX) as usize;
        if old_size > MAX_REFCOUNT {
            abort();
        }

        Self::from_inner(self.ptr)
    }
}

impl<T> Deref for Drc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T> Drop for Drc<T> {
    fn drop(&mut self) {
        if self.inner().strong.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        self.inner().strong.load(Ordering::Acquire);

        unsafe {
            self.drop_slow();
        }
    }
}
