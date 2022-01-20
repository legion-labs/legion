use lgn_ecs::world::World;
use lgn_tasks::{ComputeTaskPool, TaskPoolBuilder};
use lgn_tracing::trace;

/// Helper for configuring and creating the default task pools. For end-users who want full control,
/// insert the default task pools into the resource map manually. If the pools are already inserted,
/// this helper will do nothing.
#[derive(Clone)]
pub struct DefaultTaskPoolOptions {
    /// If the number of physical cores is less than min_total_threads, force using
    /// min_total_threads
    pub min_total_threads: usize,
    /// If the number of physical cores is greater than max_total_threads, force using
    /// max_total_threads
    pub max_total_threads: usize,
}

impl Default for DefaultTaskPoolOptions {
    fn default() -> Self {
        // By default, use however many cores are available on the system
        Self::new(1, std::usize::MAX)
    }
}

impl DefaultTaskPoolOptions {
    /// Create a configuration with a specified range of thread count.
    pub fn new(min_total_threads: usize, max_total_threads: usize) -> Self {
        Self {
            min_total_threads,
            max_total_threads,
        }
    }

    /// Inserts the default thread pools into the given resource map based on the configured values
    pub fn create_default_pools(&self, world: &mut World) {
        let total_threads =
            lgn_tasks::logical_core_count().clamp(self.min_total_threads, self.max_total_threads);

        if !world.contains_resource::<ComputeTaskPool>() {
            // Use 100%  of threads for compute
            trace!("Assigning {} cores to compute task pool", total_threads);
            world.insert_resource(ComputeTaskPool(
                TaskPoolBuilder::default()
                    .num_threads(total_threads)
                    .thread_name("Compute Task Pool".to_string())
                    .build(),
            ));
        }
    }
}
