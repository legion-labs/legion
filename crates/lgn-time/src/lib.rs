//! Legion Time
//!
//! TODO: write documentation.

// crate-specific lint exceptions:
//#![allow()]

mod auto_destruct;
mod fixed_timestep;
mod stopwatch;
#[allow(clippy::module_inception)]
mod time;
mod timer;

pub use auto_destruct::AutoDestruct;
pub use fixed_timestep::*;
pub use stopwatch::*;
pub use time::*;
pub use timer::*;

pub mod prelude {
    //! The Legion Time Prelude.
    #[doc(hidden)]
    pub use crate::{AutoDestruct, Time, Timer};
}

use crate::auto_destruct::tick_auto_destruct;
use lgn_app::prelude::*;
use lgn_ecs::prelude::*;

/// Adds time functionality to Apps.
#[derive(Default)]
pub struct TimePlugin;

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemLabel)]
/// Updates the elapsed time. Any system that interacts with [Time] component should run after
/// this.
pub struct TimeSystem;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Time>()
            .init_resource::<FixedTimesteps>()
            // time system is added as an "exclusive system" to ensure it runs before other systems
            // in CoreStage::First
            .add_system_to_stage(
                CoreStage::First,
                time_system.exclusive_system().at_start().label(TimeSystem),
            )
            .add_system_to_stage(CoreStage::Update, tick_auto_destruct);
    }
}

fn time_system(mut time: ResMut<'_, Time>) {
    time.update();
}
