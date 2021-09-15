use std::{
    future::Future,
    sync::{Arc, Mutex, Weak},
};

use super::operation::{
    AsyncOperation, AsyncOperationCanceller, AsyncOperationError, AsyncOperationResult,
};

use retain_mut::RetainMut;
use tokio::runtime::{Builder, Runtime};

// Wraps a tokio::runtime::Runtime to make it compatible with the 'systems'
// system.
pub struct TokioAsyncRuntime {
    tokio_runtime: Runtime,
    wrappers: Vec<Box<dyn TokioFutureWrapperAsyncResult>>,
}

impl TokioAsyncRuntime {
    fn spawn_in_tokio_thread_pool<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + Sync + 'static,
    {
        self.tokio_runtime.spawn(future);
    }
}

impl Default for TokioAsyncRuntime {
    fn default() -> Self {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();

        Self {
            tokio_runtime: rt,
            wrappers: vec![],
        }
    }
}

impl TokioAsyncRuntime {
    pub fn start<F>(&mut self, future: F) -> AsyncOperation<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + Sync + 'static,
    {
        let result = Arc::new(Mutex::new(None));

        let (canceller, cancelled) = tokio::sync::mpsc::unbounded_channel();
        let canceller = TokioFutureCanceller::new(canceller);
        let wrapper = Box::new(TokioFutureWrapper::new(
            self,
            future,
            Arc::downgrade(&result),
            cancelled,
        ));

        self.wrappers.push(wrapper);

        AsyncOperation::new(result, Box::new(canceller))
    }

    pub fn poll(&mut self) {
        self.wrappers
            .retain_mut(|wrapper| wrapper.poll().is_polling());
    }
}
pub enum TokioFutureWrapperPoll {
    Polling,
    Ready,
}

impl TokioFutureWrapperPoll {
    fn is_polling(&self) -> bool {
        matches!(self, &Self::Polling)
    }
}

trait TokioFutureWrapperAsyncResult: Send + Sync {
    fn poll(&mut self) -> TokioFutureWrapperPoll;
}

struct TokioFutureWrapper<T> {
    receiver: tokio::sync::oneshot::Receiver<Result<T, AsyncOperationError>>,
    result: Weak<Mutex<AsyncOperationResult<T>>>,
}

impl<T: Send + Sync + 'static> TokioFutureWrapper<T> {
    fn new<F>(
        rt: &TokioAsyncRuntime,
        future: F,
        result: Weak<Mutex<AsyncOperationResult<T>>>,
        mut cancelled: tokio::sync::mpsc::UnboundedReceiver<AsyncOperationError>,
    ) -> Self
    where
        F: Future<Output = T> + Send + 'static,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let wrapper = Self { receiver, result };

        rt.spawn_in_tokio_thread_pool(async move {
            let fut = async move {
                tokio::select! {
                    err = cancelled.recv() => {
                        cancelled.close();

                        Err(err.unwrap_or(AsyncOperationError::Dropped))
                    }
                    value = future => {
                        Ok(value)
                    }
                }
            };

            let _ = sender.send(fut.await);
        });

        wrapper
    }
}

impl<T: Send + Sync + 'static> TokioFutureWrapperAsyncResult for TokioFutureWrapper<T> {
    fn poll(&mut self) -> TokioFutureWrapperPoll {
        if let Ok(v) = self.receiver.try_recv() {
            if let Some(result) = self.result.upgrade() {
                let mut result = result.lock().unwrap();
                *result = Some(v);
            }

            // It doesn't matter that we could actually set the value in the
            // related AsyncOperation or not: we will only get that value once
            // and should never be polled again.
            return TokioFutureWrapperPoll::Ready;
        }

        TokioFutureWrapperPoll::Polling
    }
}

struct TokioFutureCanceller {
    canceller: tokio::sync::mpsc::UnboundedSender<AsyncOperationError>,
}

impl TokioFutureCanceller {
    fn new(canceller: tokio::sync::mpsc::UnboundedSender<AsyncOperationError>) -> Self {
        Self { canceller }
    }
}

impl AsyncOperationCanceller for TokioFutureCanceller {
    fn cancel(&self) {
        let _ = self.canceller.send(AsyncOperationError::Cancelled);
    }
}
