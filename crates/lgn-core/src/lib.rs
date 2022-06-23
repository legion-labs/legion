//! Legion Core
//!
//! This crate provides core functionality for Legion Engine.

// crate-specific lint exceptions:
#![allow(clippy::needless_pass_by_value)]
#![warn(missing_docs)]

mod memory;
mod name;
mod task_pool_options;

pub use bytemuck::{bytes_of, cast_slice, Pod, Zeroable};
pub use memory::*;
pub use name::*;
pub use task_pool_options::DefaultTaskPoolOptions;

pub mod prelude {
    //! The Legion Core Prelude.
    #[doc(hidden)]
    pub use crate::{DefaultTaskPoolOptions, Name};
}

use lgn_app::prelude::*;

/// Adds core functionality to Apps.
#[derive(Default)]
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // Setup the default legion task pools
        app.world
            .get_resource::<DefaultTaskPoolOptions>()
            .cloned()
            .unwrap_or_default()
            .create_default_pools();

        let bump_allocator_pool = BumpAllocatorPool::new();
        app.insert_resource(bump_allocator_pool)
            .add_system_to_stage(CoreStage::First, begin_frame)
            .add_system_to_stage(CoreStage::Last, end_frame);
    }
}
