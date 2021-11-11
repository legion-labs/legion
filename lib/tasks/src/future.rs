use std::future::Future;

use futures_lite::future;

#[inline]
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    future::block_on(future)
}

#[inline]
pub fn poll_once<T, F>(f: F) -> future::PollOnce<F>
where
    F: Future<Output = T>,
{
    future::poll_once(f)
}
