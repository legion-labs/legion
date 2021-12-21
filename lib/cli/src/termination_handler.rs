use anyhow::Context;
use lazy_static::lazy_static;
use lgn_telemetry::{info, warn};
use tokio::sync::RwLock;

use super::async_reverse_single_lock::AsyncReverseSingleLock;

lazy_static! {
    static ref TERMINATION_RWLOCK: RwLock<()> = RwLock::new(());
    static ref TERMINATION_LOCK: AsyncReverseSingleLock<'static> =
        AsyncReverseSingleLock::new(&TERMINATION_RWLOCK);
}

/// `ReverseSingleLock` is a lock that is created locked and that can only be unlocked once.
pub struct AsyncTerminationHandler {
    rwlock: &'static AsyncReverseSingleLock<'static, ()>,
}

impl AsyncTerminationHandler {
    pub fn new() -> anyhow::Result<Self> {
        Self::new_with_lock(&TERMINATION_LOCK)
    }

    fn new_with_lock(rwlock: &'static AsyncReverseSingleLock<'static, ()>) -> anyhow::Result<Self> {
        ctrlc::set_handler(move || {
            if rwlock.unlock().is_some() {
                info!("the termination handler was just triggered");
            } else {
                warn!("the termination handler was just re-triggered");
            }
        })
        .context("setting up termination handler")?;

        info!("a termination handler was setup successfully");

        Ok(Self { rwlock })
    }

    pub fn try_wait(&self) -> Option<()> {
        self.rwlock.try_wait()
    }

    pub async fn wait(&self) {
        self.rwlock.wait().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_termination_handler() -> anyhow::Result<()> {
        let handler = AsyncTerminationHandler::new()?;

        assert!(handler.try_wait().is_none());

        tokio::select! {
            biased;
            _ = handler.wait() => panic!("wait() should not have returned"),
            _ = async {} => {},
        };

        Ok(())
    }
}
