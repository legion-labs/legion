use std::future::Future;

use futures_lite::future;

#[inline]
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    future::block_on(future)
}
