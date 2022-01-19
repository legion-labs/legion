//! Legion Core
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
#![allow(clippy::needless_pass_by_value)]
#![warn(missing_docs)]

mod float_ord;
mod name;
mod task_pool_options;
mod time;

pub use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
pub use float_ord::*;
pub use name::*;
pub use task_pool_options::DefaultTaskPoolOptions;
pub use time::*;

pub mod prelude {
    //! The Bevy Core Prelude.
    #[doc(hidden)]
    pub use crate::{DefaultTaskPoolOptions, Name, Time, Timer};
}

use lgn_app::prelude::*;
use lgn_ecs::{
    schedule::{ExclusiveSystemDescriptorCoercion, SystemLabel},
    system::IntoExclusiveSystem,
};

/// Adds core functionality to Apps.
#[derive(Default)]
pub struct CorePlugin;

/// A `SystemLabel` enum for ordering systems relative to core Bevy systems.
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
            .unwrap_or_default()
            .create_default_pools(&mut app.world);

        app.init_resource::<Time>()
            .init_resource::<FixedTimesteps>()
            // time system is added as an "exclusive system" to ensure it runs before other systems
            // in CoreStage::First
            .add_system_to_stage(
                CoreStage::First,
                time_system.exclusive_system().label(CoreSystem::Time),
            );
    }
}
