#![allow(unsafe_code)]

use std::{any::Any, cell::RefCell, ops::Deref, process::abort, ptr::NonNull, sync::{Arc, Mutex, atomic::{self, Ordering}}};

#[derive(Debug, Clone)]
pub struct DeferredDropper {
    inner: Arc<Mutex<RefCell<DeferredDropperInner>>>
}

impl DeferredDropper {
    pub fn new(render_frame_capacity: usize) -> Self {
        DeferredDropper {
            inner: Arc::new(Mutex::new( RefCell::new( 
                DeferredDropperInner{                    
                    render_frame_capacity,
                    render_frame_index: 0,
                    buckets: (0..render_frame_capacity).map( |_x| ObjectBucket( Vec::new() ) ).collect()
                } 
            ) ))
        }
    }

    pub fn new_drc<T>(&self, data: T) -> Drc<T> {
        Drc::new(self.clone(), data)
    }

    pub fn defer_drop(&self, object: Box<dyn Any>) {
        let guard = self.inner.lock().unwrap();        
        let mut inner = guard.borrow_mut();
        let render_frame_index = inner.render_frame_index;
        inner.buckets[render_frame_index].0.push(object);    
    }

    pub fn flush(&self) {
        let guard = self.inner.lock().unwrap();        
        let mut inner = guard.borrow_mut();
        let next_render_frame = (inner.render_frame_index + 1)%inner.render_frame_capacity;
        inner.buckets[next_render_frame].0.drain(..);    
        inner.render_frame_index = next_render_frame;
    }
}

#[derive(Debug)]
pub struct ObjectBucket (pub Vec<Box<dyn Any>>);

#[derive(Debug)]
struct DeferredDropperInner {
    pub render_frame_capacity: usize,
    pub render_frame_index: usize,
    pub buckets: Vec<ObjectBucket>,    
}

unsafe impl Send for DeferredDropperInner {}

unsafe impl Sync for DeferredDropperInner {}

#[derive(Debug)]
struct DrcInner<T> {
    strong: atomic::AtomicUsize,
    dropper: DeferredDropper,
    data: T,
}

#[derive(Debug)]
pub struct Drc<T: 'static>  {    
    ptr: NonNull<DrcInner<T>>,
}

impl<T> Drc<T> {
    
    pub fn new(dropper: DeferredDropper, data: T) -> Self {
        let x = Box::new(
            DrcInner {
                strong: atomic::AtomicUsize::new(1),            
                dropper,
                data,
            }
        );

        Self::from_inner(Box::leak(x).into())        
    }

    fn from_inner(ptr: NonNull<DrcInner<T>>) -> Self {
        Self {             
            ptr
        }
    }   
    
    fn inner(&self) -> &DrcInner<T> {                       
        unsafe { self.ptr.as_ref() }
    }

    unsafe fn drop_slow(&mut self) {

        let boxed = Box::<dyn Any>::from_raw(self.ptr.as_ptr());
        let dropper = &self.ptr.as_ref().dropper;
        dropper.defer_drop(boxed);
    }
}

unsafe impl<T> Send for Drc<T> where T: Send {}

unsafe impl<T> Sync for Drc<T> where T: Sync {}

impl<T> Clone for Drc<T> {
    fn clone(&self) -> Drc<T> {         
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