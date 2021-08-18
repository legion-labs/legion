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
#[cfg(feature = "legion-reflect")]
use legion_ecs::entity::Entity;
use legion_ecs::{
    schedule::{ExclusiveSystemDescriptorCoercion, SystemLabel},
    system::IntoExclusiveSystem,
};
#[cfg(feature = "legion-reflect")]
use legion_utils::HashSet;
#[cfg(feature = "legion-reflect")]
use std::ops::Range;

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
            .init_resource::<FixedTimesteps>();

        #[cfg(feature = "legion-reflect")]
        app.register_type::<HashSet<String>>()
            .register_type::<Option<String>>()
            .register_type::<Entity>()
            .register_type::<Name>()
            .register_type::<Labels>()
            .register_type::<Range<f32>>()
            .register_type::<Timer>();

        // time system is added as an "exclusive system" to ensure it runs before other systems
        // in CoreStage::First
        app.add_system_to_stage(
            CoreStage::First,
            time_system.exclusive_system().label(CoreSystem::Time),
        )
        .add_startup_system_to_stage(StartupStage::PostStartup, entity_labels_system)
        .add_system_to_stage(CoreStage::PostUpdate, entity_labels_system);

        #[cfg(feature = "legion-reflect")]
        register_rust_types(app);
        #[cfg(feature = "legion-reflect")]
        register_math_types(app);
    }
}

#[cfg(feature = "legion-reflect")]
fn register_rust_types(app: &mut App) {
    app.register_type::<bool>()
        .register_type::<u8>()
        .register_type::<u16>()
        .register_type::<u32>()
        .register_type::<u64>()
        .register_type::<u128>()
        .register_type::<usize>()
        .register_type::<i8>()
        .register_type::<i16>()
        .register_type::<i32>()
        .register_type::<i64>()
        .register_type::<i128>()
        .register_type::<isize>()
        .register_type::<f32>()
        .register_type::<f64>()
        .register_type::<String>()
        .register_type::<Option<String>>();
}

#[cfg(feature = "legion-reflect")]
fn register_math_types(app: &mut App) {
    app.register_type::<legion_math::IVec2>()
        .register_type::<legion_math::IVec3>()
        .register_type::<legion_math::IVec4>()
        .register_type::<legion_math::UVec2>()
        .register_type::<legion_math::UVec3>()
        .register_type::<legion_math::UVec4>()
        .register_type::<legion_math::Vec2>()
        .register_type::<legion_math::Vec3>()
        .register_type::<legion_math::Vec4>()
        .register_type::<legion_math::Mat3>()
        .register_type::<legion_math::Mat4>()
        .register_type::<legion_math::Quat>();
}
