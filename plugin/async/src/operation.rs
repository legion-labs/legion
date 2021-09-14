use std::{
    error::Error,
    sync::{Arc, Mutex},
};

pub type AsyncOperationResult<T> = Option<Result<T, Box<dyn Error + Send + Sync>>>;

// Represents an online operation running in a separate thread pool, that can be
// polled for completion.
pub struct AsyncOperation<T> {
    result: Arc<Mutex<AsyncOperationResult<T>>>,
}

impl<T> AsyncOperation<T> {
    pub fn new(result: Arc<Mutex<AsyncOperationResult<T>>>) -> AsyncOperation<T> {
        AsyncOperation::<T> { result }
    }
}

impl<T: Send + 'static> AsyncOperation<T> {
    pub fn get_result(&self) -> Option<Result<T, Box<dyn Error + Send + Sync>>> {
        self.result.lock().unwrap().take()
    }
}
