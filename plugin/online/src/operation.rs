use std::{error::Error, future::Future, sync::Arc, task::Poll};

// An OperationStatus represents the current status of an operation.
pub enum OnlineOperationStatus<T> {
    Idle,
    Started,
    Failed(&'static dyn Error),
    Completed(Arc<T>),
}

use OnlineOperationStatus::*;

use crate::{OnlineFuture, OnlineRuntime};

// Represents an online operation running in a separate thread pool, that can be
// polled for completion.
#[derive(Default)]
pub struct OnlineOperation<T> {
    future: Option<OnlineFuture<T>>,
    result: Option<Arc<T>>,
}

impl<T: Send + 'static> OnlineOperation<T> {
    pub fn new_started<Runtime: OnlineRuntime, F: Future<Output = T> + Send + 'static>(
        rt: &Runtime,
        future: F,
    ) -> OnlineOperation<T> {
        let mut op = OnlineOperation::<T> {
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

    pub fn start_with<Runtime: OnlineRuntime, F: Future<Output = T> + Send + 'static>(
        &mut self,
        rt: &Runtime,
        future: F,
    ) -> Result<(), OnlineOperationAlreadyStartedError> {
        match self.future {
            None => {
                self.future = Some(rt.start(future));

                Ok(())
            }
            Some(_) => Err(OnlineOperationAlreadyStartedError),
        }
    }

    pub fn restart_with<Runtime: OnlineRuntime, F: Future<Output = T> + Send + 'static>(
        &mut self,
        rt: &Runtime,
        future: F,
    ) {
        self.reset();
        self.start_with(rt, future).unwrap()
    }

    pub fn poll<Runtime: OnlineRuntime>(&mut self, rt: &Runtime) -> OnlineOperationStatus<T> {
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
pub struct OnlineOperationAlreadyStartedError;
