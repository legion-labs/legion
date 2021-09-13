use std::cell::RefCell;
use std::future::Future;
use std::sync::mpsc::{sync_channel, Receiver};

// Wraps a tokio::runtime::Runtime to make it compatible with the 'systems'
// system.
pub struct Runtime {
    tokio_runtime: tokio::runtime::Runtime,
}

impl Default for Runtime {
    fn default() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Runtime { tokio_runtime: rt }
    }
}

impl Runtime {
    pub fn spawn<F>(&self, f: F) -> Result<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (setter, result) = Result::new();

        self.tokio_runtime.spawn(async {
            setter(f.await);
        });

        result
    }
}

// Represents an online result that can be polled for values.
pub struct Result<T: Send + 'static> {
    receiver: Receiver<T>,
    value: RefCell<Option<T>>,
}

unsafe impl<T: Send + 'static> Sync for Result<T> {}

impl<T: Send + 'static> Result<T> {
    pub fn new() -> (impl FnOnce(T) -> (), Result<T>) {
        let (sender, receiver) = sync_channel(1);
        let setter = move |t| sender.send(t).unwrap();

        (
            setter,
            Result {
                receiver: receiver,
                value: RefCell::new(None),
            },
        )
    }

    fn receive_if_unset(&self) {
        if self.value.borrow().is_none() {
            if let Ok(v) = self.receiver.try_recv() {
                *self.value.borrow_mut() = Some(v);
            }
        }
    }

    pub fn is_set(&self) -> bool {
        self.receive_if_unset();
        self.value.borrow().is_some()
    }
}

impl<T: Send + 'static + Clone> Result<T> {
    pub fn get(&self) -> Option<T> {
        self.receive_if_unset();
        self.value.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_set_and_get() {
        let (setter, ret) = Result::new();

        assert!(ret.get().is_none());
        setter(42);
        assert!(ret.get().unwrap() == 42);
    }
}
