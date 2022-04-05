use std::{ops::Deref, sync::Arc};

use tokio::sync::{broadcast::Receiver, Mutex};

/// Easy to share, referenced counted version of Tokio's [`Receiver`].
/// Can be cloned safely and will dereference to the internal [`Mutex`].
pub struct SharedUnboundedReceiver<T>(Arc<Mutex<Receiver<T>>>);

impl<T> SharedUnboundedReceiver<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        Self(Arc::new(Mutex::new(receiver)))
    }
}

impl<T> Clone for SharedUnboundedReceiver<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for SharedUnboundedReceiver<T> {
    type Target = Mutex<Receiver<T>>;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> From<Receiver<T>> for SharedUnboundedReceiver<T> {
    fn from(receiver: Receiver<T>) -> Self {
        Self::new(receiver)
    }
}
