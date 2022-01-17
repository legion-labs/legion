use std::sync::Mutex;

use tokio::sync::{RwLock, RwLockWriteGuard};

/// `AsyncReverseSingleLock` is a lock that is created locked and that can only
/// be unlocked once.
pub(crate) struct AsyncReverseSingleLock<'a, T = ()> {
    rwlock: &'a RwLock<T>,
    wguard: Mutex<Option<RwLockWriteGuard<'a, T>>>,
}

impl<'a, T: 'a> AsyncReverseSingleLock<'a, T> {
    pub(crate) fn new(rwlock: &'a RwLock<T>) -> Self {
        Self {
            rwlock,
            wguard: Mutex::new(rwlock.try_write().ok()),
        }
    }

    /// Unlock the lock if it was previously locked, returning true if the lock
    /// was previously locked.
    pub(crate) fn unlock(&self) -> bool {
        self.wguard
            .lock()
            .expect("failed to acquire rwlock guard mutex")
            .take()
            .is_some()
    }

    // Try to wait for the lock to be unlocked.
    pub(crate) fn try_wait(&self) -> bool {
        self.rwlock.try_read().is_ok()
    }

    // Wait for the lock to be unlocked.
    pub(crate) async fn wait(&self) {
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

        assert!(!lock.try_wait());

        tokio::select! {
            biased;
            _ = lock.wait() => panic!("wait() should not have returned"),
            _ = async {} => {},
        };

        assert!(lock.unlock());
        assert!(!lock.unlock());

        assert!(lock.try_wait());

        tokio::select! {
            biased;
            _ = lock.wait() => {},
            _ = async {} => panic!("wait() should have returned"),
        };

        Ok(())
    }
}
