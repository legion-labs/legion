use std::sync::Mutex;

use tokio::sync::{RwLock, RwLockWriteGuard};

pub(crate) struct AsyncReverseSingleLock<'a, T = ()> {
    rwlock: &'a RwLock<T>,
    wguard: Mutex<Option<RwLockWriteGuard<'a, T>>>,
}

impl<'a, T: 'a> AsyncReverseSingleLock<'a, T> {
    pub fn new(rwlock: &'a RwLock<T>) -> Self {
        Self {
            rwlock,
            wguard: Mutex::new(rwlock.try_write().ok()),
        }
    }

    /// Unlock the lock if it was previously locked, returning a non-empty option if the lock was
    /// previously locked.
    pub fn unlock(&self) -> Option<()> {
        self.wguard
            .lock()
            .expect("failed to acquire rwlock guard mutex")
            .take()
            .map(|_| ())
    }

    // Try to wait for the lock to be unlocked.
    pub fn try_wait(&self) -> Option<()> {
        self.rwlock.try_read().map(|_| ()).ok()
    }

    // Wait for the lock to be unlocked.
    pub async fn wait(&self) {
        self.rwlock.read().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_reverse_single_lock() -> anyhow::Result<()> {
        let rwlock = RwLock::new(());
        let lock = AsyncReverseSingleLock::new(&rwlock);

        assert!(lock.try_wait().is_none());

        tokio::select! {
            biased;
            _ = lock.wait() => panic!("wait() should not have returned"),
            _ = async {} => {},
        };

        assert!(lock.unlock().is_some());
        assert!(lock.unlock().is_none());

        assert!(lock.try_wait().is_some());

        tokio::select! {
            biased;
            _ = lock.wait() => {},
            _ = async {} => panic!("wait() should have returned"),
        };

        Ok(())
    }
}
