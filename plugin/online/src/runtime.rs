use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::runtime::{Builder, Runtime};

pub trait OnlineRuntime {
    fn start<F>(&self, future: F) -> OnlineFuture<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + 'static;
}

// Wraps a tokio::runtime::Runtime to make it compatible with the 'systems'
// system.
pub struct TokioOnlineRuntime {
    tokio_runtime: Runtime,
}

impl Default for TokioOnlineRuntime {
    fn default() -> Self {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();

        TokioOnlineRuntime { tokio_runtime: rt }
    }
}

impl OnlineRuntime for TokioOnlineRuntime {
    fn start<F>(&self, future: F) -> OnlineFuture<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + 'static,
    {
        let (sender, res) = OnlineFuture::new();

        // Dispatch the specified future in tokio's thread-pool. Once it
        // completes, were are responsible for completing the OnlineFuture
        // accordingly. This is what OnlineRuntimes do.
        self.tokio_runtime.spawn(async move {
            let _ = sender.send(future.await);
        });

        res
    }
}

pub struct OnlineFuture<T> {
    receiver: tokio::sync::oneshot::Receiver<T>,
}

impl<T> OnlineFuture<T> {
    fn new() -> (tokio::sync::oneshot::Sender<T>, OnlineFuture<T>) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let future = OnlineFuture { receiver };

        (sender, future)
    }
}

impl<T> Future for OnlineFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut().receiver.try_recv() {
            Ok(v) => Poll::Ready(v),
            Err(_) => {
                // We actually want to poll all the time.
                //
                // TODO: This is good enough for now but we might want to make that
                // smarter in the future.
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}
