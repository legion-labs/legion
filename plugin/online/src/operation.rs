use std::{
    error::Error,
    sync::Arc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

// An OperationStatus represents the current status of an operation.
pub enum OnlineOperationStatus<T> {
    Idle,
    Started,
    Cancelled(&'static dyn Error),
    Completed(Arc<T>),
}

use futures_lite::FutureExt;
use OnlineOperationStatus::*;

use crate::OnlineFuture;

// Represents an online operation running in a separate thread pool, that can be
// polled for completion.
#[derive(Default)]
pub struct OnlineOperation<T> {
    future: Option<OnlineFuture<T>>,
    result: Option<Arc<T>>,
}

impl<T> OnlineOperation<T> {
    pub fn new_started(future: OnlineFuture<T>) -> OnlineOperation<T> {
        let mut op = OnlineOperation::<T> {
            future: None,
            result: None,
        };

        let _ = op.start_with(future);

        op
    }

    pub fn reset(&mut self) {
        self.future = None;
        self.result = None;
    }

    pub fn start_with(
        &mut self,
        future: OnlineFuture<T>,
    ) -> Result<(), OnlineOperationAlreadyStartedError> {
        match self.future {
            None => {
                self.future = Some(future);

                Ok(())
            }
            Some(_) => Err(OnlineOperationAlreadyStartedError),
        }
    }

    pub fn restart_with(&mut self, future: OnlineFuture<T>) {
        self.reset();
        self.start_with(future).unwrap()
    }

    pub fn poll(&mut self) -> OnlineOperationStatus<T> {
        // If we already have a result in store, let's return that: our job is
        // done.
        if let Some(v) = &self.result {
            return Completed(Arc::clone(v));
        }

        // If we have a future, we must check whether it is actually ready or not.
        if let Some(future) = &mut self.future {
            let raw = RawWaker::new(std::ptr::null(), &VTABLE);
            let waker = unsafe { Waker::from_raw(raw) };
            let mut cx = Context::from_waker(&waker);

            return match future.poll(&mut cx) {
                Poll::Pending => Started,
                Poll::Ready(v) => {
                    let v = Arc::new(v);
                    self.result = Some(Arc::clone(&v));
                    Completed(v)
                }
            };
        }

        // We have no result and no associated future: we were never started.
        Idle
    }
}

// Indicates that an operation was already started and could not be restarted.
#[derive(Debug, Clone)]
pub struct OnlineOperationAlreadyStartedError;

fn do_nothing(_ptr: *const ()) {}

fn clone(ptr: *const ()) -> RawWaker {
    RawWaker::new(ptr, &VTABLE)
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, do_nothing, do_nothing, do_nothing);
