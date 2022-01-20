use std::ops::{Bound, RangeBounds};

use lgn_ecs::world::World;
use lgn_tasks::{ComputeTaskPool, TaskPoolBuilder};
use lgn_tracing::info;

/// Helper for configuring and creating the default task pools. For end-users who want full control,
/// insert the default task pools into the resource map manually. If the pools are already inserted,
/// this helper will do nothing.
#[derive(Clone)]
pub struct DefaultTaskPoolOptions {
    /// If the number of physical cores is less than min_total_threads, force using
    /// min_total_threads
    pub min_total_threads: Bound<usize>,
    /// If the number of physical cores is greater than max_total_threads, force using
    /// max_total_threads
    pub max_total_threads: Bound<usize>,
}

impl Default for DefaultTaskPoolOptions {
    fn default() -> Self {
        // By default, use however many cores are available on the system
        Self::new(1..)
    }
}

impl DefaultTaskPoolOptions {
    /// Create a configuration with a specified range of thread count.
    pub fn new(total_thread_count_range: impl RangeBounds<usize>) -> Self {
        Self {
            min_total_threads: total_thread_count_range.start_bound().cloned(),
            max_total_threads: total_thread_count_range.end_bound().cloned(),
        }
    }

    /// Inserts the default thread pools into the given resource map based on the configured values
    pub fn create_default_pools(&self, world: &mut World) {
        let mut total_threads = lgn_tasks::logical_core_count();
        match self.min_total_threads {
            Bound::Included(min) => {
                total_threads = total_threads.max(min);
            }
            Bound::Excluded(min) => {
                total_threads = total_threads.max(min + 1);
            }
            Bound::Unbounded => {}
        }
        match self.max_total_threads {
            Bound::Included(max) => {
                total_threads = total_threads.min(max);
            }
            Bound::Excluded(max) => {
                total_threads = total_threads.min(max - 1);
            }
            Bound::Unbounded => {}
        }

        if !world.contains_resource::<ComputeTaskPool>() {
            // Use 100%  of threads for compute
            info!("Assigning {} cores to compute task pool", total_threads);
            world.insert_resource(ComputeTaskPool(
                TaskPoolBuilder::default()
                    .num_threads(total_threads)
                    .thread_name("Compute Task Pool".to_string())
                    .build(),
            ));
        }
    }
}
