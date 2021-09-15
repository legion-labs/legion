use std::{
    error::Error,
    sync::{Arc, Mutex},
};

// Represents an async operation result.
//
// A lack of value indicates that the associated AsyncOperation is not done yet.
pub type AsyncOperationResult<T> = Option<Result<T, Box<dyn Error + Send + Sync>>>;

// Represents an async operation running in a separate thread pool, that can be
// polled for completion.
pub struct AsyncOperation<T> {
    result: Arc<Mutex<AsyncOperationResult<T>>>,
}

impl<T> AsyncOperation<T> {
    pub fn new(result: Arc<Mutex<AsyncOperationResult<T>>>) -> Self {
        Self { result }
    }
}

impl<T: Send + 'static> AsyncOperation<T> {
    pub fn take_result(&self) -> Option<Result<T, Box<dyn Error + Send + Sync>>> {
        self.result.lock().unwrap().take()
    }
}
