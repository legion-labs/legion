mod bytes;
mod float_ord;
mod label;
mod name;
mod task_pool_options;
mod time;

pub use bytes::*;
pub use float_ord::*;
pub use label::*;
pub use name::*;
pub use task_pool_options::DefaultTaskPoolOptions;
pub use time::*;

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{DefaultTaskPoolOptions, EntityLabels, Labels, Name, Time, Timer};
}

use legion_app::prelude::*;
use legion_ecs::{
    schedule::{ExclusiveSystemDescriptorCoercion, SystemLabel},
    system::IntoExclusiveSystem,
};

/// Adds core functionality to Apps.
#[derive(Default)]
pub struct CorePlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
pub enum CoreSystem {
    /// Updates the elapsed time. Any system that interacts with [Time] component should run after
    /// this.
    Time,
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Setup the default legion task pools
        app.world
            .get_resource::<DefaultTaskPoolOptions>()
            .cloned()
            .unwrap_or_else(DefaultTaskPoolOptions::default)
            .create_default_pools(&mut app.world);

        app.init_resource::<Time>()
            .init_resource::<EntityLabels>()
            .init_resource::<FixedTimesteps>()
            // time system is added as an "exclusive system" to ensure it runs before other systems
            // in CoreStage::First
            .add_system_to_stage(
                CoreStage::First,
                time_system.exclusive_system().label(CoreSystem::Time),
            )
            .add_startup_system_to_stage(StartupStage::PostStartup, entity_labels_system)
            .add_system_to_stage(CoreStage::PostUpdate, entity_labels_system);
    }
}
