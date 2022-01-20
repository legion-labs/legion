//! Definitions for a few common task pools that we want. Generally the determining factor for what
//! kind of work should go in each pool is latency requirements.
//!
//! For CPU-intensive work (tasks that generally spin until completion) we have a standard
//! [`ComputeTaskPool`].

use std::ops::Deref;

use super::TaskPool;

/// A newtype for a task pool for CPU-intensive work that must be completed to
/// deliver the next frame
#[derive(Clone, Debug)]
pub struct ComputeTaskPool(pub TaskPool);

impl Deref for ComputeTaskPool {
    type Target = TaskPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
