//! Definitions for a few common task pools that we want. Generally the determining factor for what
//! kind of work should go in each pool is latency requirements.
//!
//! For CPU-intensive work (tasks that generally spin until completion) we have a standard
//! [`ComputeTaskPool`].

use std::ops::Deref;

use once_cell::sync::OnceCell;

use super::TaskPool;

static COMPUTE_TASK_POOL: OnceCell<ComputeTaskPool> = OnceCell::new();

/// A newtype for a task pool for CPU-intensive work that must be completed to
/// deliver the next frame
#[derive(Debug)]
pub struct ComputeTaskPool(TaskPool);

impl ComputeTaskPool {
    /// Initializes the global [`ComputeTaskPool`] instance.
    pub fn init(f: impl FnOnce() -> TaskPool) -> &'static Self {
        COMPUTE_TASK_POOL.get_or_init(|| Self(f()))
    }

    /// Gets the global [`ComputeTaskPool`] instance.
    ///
    /// # Panics
    /// Panics if no pool has been initialized yet.
    pub fn get() -> &'static Self {
        COMPUTE_TASK_POOL.get().expect(
            "A ComputeTaskPool has not been initialized yet. Please call \
                    ComputeTaskPool::init beforehand.",
        )
    }
}

impl Deref for ComputeTaskPool {
    type Target = TaskPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
