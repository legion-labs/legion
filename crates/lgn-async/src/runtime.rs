use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
};

use lgn_tasks::logical_core_count;
use lgn_tracing::{
    dispatch::{flush_thread_buffer, init_thread_stream, on_begin_scope, on_end_scope},
    max_lod,
    spans::ThreadSpanMetadata,
    Lod,
};
use retain_mut::RetainMut;
use tokio::runtime::{Builder, Handle, Runtime};

use super::operation::{AsyncOperation, AsyncOperationError, AsyncOperationResult};

// Wraps a tokio::runtime::Runtime to make it compatible with the 'systems'
// system.
pub struct TokioAsyncRuntime {
    tokio_runtime: Runtime,
    result_handlers: Vec<Box<dyn TokioFutureWrapperAsyncResult>>,
}

// newtype the Runtime handle to provide the start detached functionality
pub struct TokioAsyncRuntimeHandle(Handle);

impl Default for TokioAsyncRuntime {
    fn default() -> Self {
        static PARK_SPAN_METADATA: ThreadSpanMetadata = ThreadSpanMetadata {
            lod: Lod::Max,
            name: "tokio::busy",
            target: module_path!(),
            module_path: module_path!(),
            file: file!(),
            line: line!(),
        };
        let rt = Builder::new_multi_thread()
            .enable_all()
            .thread_name_fn(|| {
                static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
                let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
                format!("Tokio Task Pool {}", id)
            })
            .worker_threads(logical_core_count().min(4)) // take 4 thread max
            .on_thread_start(init_thread_stream)
            .on_thread_stop(flush_thread_buffer)
            .on_thread_park(|| {
                // This is inverted on purpose even if we receive these callbacks
                // the logical order park/unpark, we want to keep track of the
                // busy work on the thread.
                if PARK_SPAN_METADATA.lod <= max_lod() {
                    on_end_scope(&PARK_SPAN_METADATA);
                }
            })
            .on_thread_unpark(|| {
                if PARK_SPAN_METADATA.lod <= max_lod() {
                    on_begin_scope(&PARK_SPAN_METADATA);
                }
            })
            .max_blocking_threads(1)
            .build()
            .unwrap();

        Self {
            tokio_runtime: rt,
            result_handlers: vec![],
        }
    }
}

impl TokioAsyncRuntime {
    // Start a future on the tokio thread-pool with no implicit synchronization
    // with the main game-loop or possibility for cancellation.
    pub fn start_detached<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tokio_runtime.spawn(future);
    }

    pub fn handle(&self) -> TokioAsyncRuntimeHandle {
        TokioAsyncRuntimeHandle {
            0: self.tokio_runtime.handle().clone(),
        }
    }

    // Start a future on the tokio thread-pool that is associated to the
    // returned AsyncOperation.
    //
    // If the AsyncOperation is cancelled or dropped the future is implicitly
    // cancelled.
    pub fn start<F>(&mut self, future: F) -> AsyncOperation<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + Sync + 'static,
    {
        let result = Arc::new(Mutex::new(None));

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::unbounded_channel();
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        let result_handler = Box::new(TokioFutureWrapper::new(result_rx, Arc::downgrade(&result)));

        self.start_detached(async move {
            let fut = async move {
                tokio::select! {
                    // `biased` below ensures that the order of polling is deterministic
                    // (from top to bottom) and is required for the following reasons:
                    //
                    // - Randomization has non-zero CPU cost.
                    // - Cancellations should always win to make testing deterministic.
                    //
                    // Since the cancelled signal almost never unblocks, putting
                    // it first is fine as it won't preempt any async-polling
                    // time from the actual future.
                    biased;

                    err = cancel_rx.recv() => {
                        cancel_rx.close();

                        Err(err.unwrap_or(AsyncOperationError::Dropped))
                    }
                    value = future => {
                        Ok(value)
                    }
                }
            };

            #[allow(clippy::let_underscore_drop)]
            let _ = result_tx.send(fut.await);
        });

        self.result_handlers.push(result_handler);

        AsyncOperation::new(result, cancel_tx)
    }

    // Polls the runtime for potential completed futures, returning the number
    // of completed futures during the last call.
    pub fn poll(&mut self) -> u32 {
        let mut num_completed = 0;

        RetainMut::retain_mut(&mut self.result_handlers, |handler| {
            let is_complete = handler.try_complete();

            if is_complete {
                num_completed += 1;
            }

            !is_complete
        });

        num_completed
    }
}

impl TokioAsyncRuntimeHandle {
    pub fn start_detached<F>(&self, future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future);
    }
}

trait TokioFutureWrapperAsyncResult: Send + Sync {
    /// Returns true if the future completed.
    fn try_complete(&mut self) -> bool;
}

struct TokioFutureWrapper<T> {
    result_rx: tokio::sync::oneshot::Receiver<Result<T, AsyncOperationError>>,
    result: Weak<Mutex<AsyncOperationResult<T>>>,
}

impl<T: Send + Sync + 'static> TokioFutureWrapper<T> {
    fn new(
        result_rx: tokio::sync::oneshot::Receiver<Result<T, AsyncOperationError>>,
        result: Weak<Mutex<AsyncOperationResult<T>>>,
    ) -> Self {
        Self { result_rx, result }
    }
}

impl<T: Send + Sync + 'static> TokioFutureWrapperAsyncResult for TokioFutureWrapper<T> {
    fn try_complete(&mut self) -> bool {
        if let Ok(v) = self.result_rx.try_recv() {
            if let Some(result) = self.result.upgrade() {
                let mut result = result.lock().unwrap();
                *result = Some(v);
            }
            return true;
        }

        false
    }
}
