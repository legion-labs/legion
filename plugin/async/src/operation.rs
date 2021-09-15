use std::{
    error::Error,
    fmt,
    sync::{Arc, Mutex},
};

// Represents an async operation result.
//
// A lack of value indicates that the associated AsyncOperation is not done yet.
pub type AsyncOperationResult<T> = Option<Result<T, AsyncOperationError>>;

// An error that can happen to an AsyncOperation.
#[derive(Debug)]
pub enum AsyncOperationError {
    Cancelled,
    Dropped,
}

impl fmt::Display for AsyncOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Cancelled => write!(f, "the asynchronous operation was cancelled"),
            Self::Dropped => write!(f, "the asynchronous operation was dropped"),
        }
    }
}

impl Error for AsyncOperationError {}

pub(crate) trait AsyncOperationCanceller: Send + Sync {
    fn cancel(&self);
}

// Represents an async operation running in a separate thread pool, that can be
// polled for completion.
pub struct AsyncOperation<T> {
    result: Arc<Mutex<AsyncOperationResult<T>>>,
    canceller: Box<dyn AsyncOperationCanceller>,
}

impl<T> AsyncOperation<T> {
    pub(crate) fn new(
        result: Arc<Mutex<AsyncOperationResult<T>>>,
        canceller: Box<dyn AsyncOperationCanceller>,
    ) -> Self {
        Self { result, canceller }
    }
}

impl<T: Send + 'static> AsyncOperation<T> {
    pub fn take_result(&self) -> AsyncOperationResult<T> {
        self.result.lock().unwrap().take()
    }

    pub fn cancel(&self) {
        self.canceller.cancel();
    }
}
