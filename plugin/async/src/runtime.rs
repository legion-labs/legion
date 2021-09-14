use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use futures_lite::FutureExt;
use tokio::runtime::{Builder, Runtime};

pub trait AsyncRuntime {
    fn start<F>(&self, future: F) -> AsyncFuture<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + 'static;

    fn poll<T>(&self, future: &mut AsyncFuture<T>) -> Poll<T>;
}

// Wraps a tokio::runtime::Runtime to make it compatible with the 'systems'
// system.
pub struct TokioAsyncRuntime {
    tokio_runtime: Runtime,
}

impl Default for TokioAsyncRuntime {
    fn default() -> Self {
        let rt = Builder::new_multi_thread().enable_all().build().unwrap();

        TokioAsyncRuntime { tokio_runtime: rt }
    }
}

impl AsyncRuntime for TokioAsyncRuntime {
    fn start<F>(&self, future: F) -> AsyncFuture<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Sized + Send + 'static,
    {
        let (sender, res) = AsyncFuture::new();

        // Dispatch the specified future in tokio's thread-pool. Once it
        // completes, were are responsible for completing the OnlineFuture
        // accordingly. This is what OnlineRuntimes do.
        self.tokio_runtime.spawn(async move {
            let _ = sender.send(future.await);
        });

        res
    }

    fn poll<T>(&self, future: &mut AsyncFuture<T>) -> Poll<T> {
        let raw = RawWaker::new(std::ptr::null(), &VTABLE);
        let waker = unsafe { Waker::from_raw(raw) };
        let mut cx = Context::from_waker(&waker);

        future.poll(&mut cx)
    }
}

pub struct AsyncFuture<T> {
    receiver: tokio::sync::oneshot::Receiver<T>,
}

impl<T> AsyncFuture<T> {
    fn new() -> (tokio::sync::oneshot::Sender<T>, AsyncFuture<T>) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let future = AsyncFuture { receiver };

        (sender, future)
    }
}

impl<T> Future for AsyncFuture<T> {
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

fn do_nothing(_ptr: *const ()) {}

fn clone(ptr: *const ()) -> RawWaker {
    RawWaker::new(ptr, &VTABLE)
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, do_nothing, do_nothing, do_nothing);
