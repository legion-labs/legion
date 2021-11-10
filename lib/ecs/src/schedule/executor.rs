use downcast_rs::{impl_downcast, Downcast};
use legion_tasks::{ComputeTaskPool, TaskPool};

use crate::{archetype::ArchetypeGeneration, schedule::ParallelSystemContainer, world::World};

pub trait ParallelSystemExecutor: Downcast + Send + Sync {
    /// Called by `SystemStage` whenever `systems` have been changed.
    fn rebuild_cached_data(&mut self, systems: &[ParallelSystemContainer]);

    fn run_systems(&mut self, systems: &mut [ParallelSystemContainer], world: &mut World);
}

impl_downcast!(ParallelSystemExecutor);

pub struct SingleThreadedExecutor {
    /// Last archetypes generation observed by parallel systems.
    archetype_generation: ArchetypeGeneration,
}

impl Default for SingleThreadedExecutor {
    fn default() -> Self {
        Self {
            archetype_generation: ArchetypeGeneration::initial(),
        }
    }
}

impl ParallelSystemExecutor for SingleThreadedExecutor {
    fn rebuild_cached_data(&mut self, _: &[ParallelSystemContainer]) {}

    fn run_systems(&mut self, systems: &mut [ParallelSystemContainer], world: &mut World) {
        self.update_archetypes(systems, world);

        let compute_pool = world
            .get_resource_or_insert_with(|| ComputeTaskPool(TaskPool::default()))
            .clone();

        compute_pool.scope(|scope| {
            scope.spawn(async {
                for system in systems {
                    if system.should_run() {
                        #[cfg(feature = "trace")]
                        let system_span =
                            legion_utils::tracing::info_span!("system", name = &*system.name());
                        #[cfg(feature = "trace")]
                        let _system_guard = system_span.enter();
                        system.system_mut().run((), world).await;
                    }
                }
            });
        });
    }
}

impl SingleThreadedExecutor {
    /// Calls `system.new_archetype()` for each archetype added since the last call to
    /// [`update_archetypes`] and updates cached `archetype_component_access`.
    fn update_archetypes(&mut self, systems: &mut [ParallelSystemContainer], world: &World) {
        let archetypes = world.archetypes();
        let new_generation = archetypes.generation();
        let old_generation = std::mem::replace(&mut self.archetype_generation, new_generation);
        let archetype_index_range = old_generation.value()..new_generation.value();

        for archetype in archetypes.archetypes[archetype_index_range].iter() {
            for container in systems.iter_mut() {
                let system = container.system_mut();
                system.new_archetype(archetype);
            }
        }
    }
}
