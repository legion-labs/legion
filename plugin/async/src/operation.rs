use std::{error::Error, future::Future, sync::Arc, task::Poll};

// An OperationStatus represents the current status of an operation.
pub enum AsyncOperationStatus<T> {
    Idle,
    Started,
    Failed(&'static dyn Error),
    Completed(Arc<T>),
}

use AsyncOperationStatus::*;

use crate::{AsyncFuture, AsyncRuntime};

// Represents an online operation running in a separate thread pool, that can be
// polled for completion.
#[derive(Default)]
pub struct AsyncOperation<T> {
    future: Option<AsyncFuture<T>>,
    result: Option<Arc<T>>,
}

impl<T: Send + 'static> AsyncOperation<T> {
    pub fn new_started<Runtime: AsyncRuntime, F: Future<Output = T> + Send + 'static>(
        rt: &Runtime,
        future: F,
    ) -> AsyncOperation<T> {
        let mut op = AsyncOperation::<T> {
            future: None,
            result: None,
        };

        let _ = op.start_with(rt, future);

        op
    }

    pub fn reset(&mut self) {
        self.future = None;
        self.result = None;
    }

    pub fn start_with<Runtime: AsyncRuntime, F: Future<Output = T> + Send + 'static>(
        &mut self,
        rt: &Runtime,
        future: F,
    ) -> Result<(), AsyncOperationAlreadyStartedError> {
        match self.future {
            None => {
                self.future = Some(rt.start(future));

                Ok(())
            }
            Some(_) => Err(AsyncOperationAlreadyStartedError),
        }
    }

    pub fn restart_with<Runtime: AsyncRuntime, F: Future<Output = T> + Send + 'static>(
        &mut self,
        rt: &Runtime,
        future: F,
    ) {
        self.reset();
        self.start_with(rt, future).unwrap()
    }

    pub fn poll<Runtime: AsyncRuntime>(&mut self, rt: &Runtime) -> AsyncOperationStatus<T> {
        // If we already have a result in store, let's return that: our job is
        // done.
        if let Some(v) = &self.result {
            return Completed(Arc::clone(v));
        }

        // If we have a future, we must check whether it is actually ready or not.
        if let Some(future) = &mut self.future {
            return match rt.poll(future) {
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
pub struct AsyncOperationAlreadyStartedError;
