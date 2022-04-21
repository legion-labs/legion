//! Legion Gilrs  
//!
//! TODO: write documentation.

// crate-specific lint exceptions:

mod converter;
mod gilrs_system;

use lgn_app::{App, CoreStage, Plugin, StartupStage};
use lgn_ecs::schedule::ParallelSystemDescriptorCoercion;
use lgn_input::InputSystem;
use lgn_tracing::error;
use gilrs::GilrsBuilder;
use gilrs_system::{gilrs_event_startup_system, gilrs_event_system};

#[derive(Default)]
pub struct GilrsPlugin;

impl Plugin for GilrsPlugin {
    fn build(&self, app: &mut App) {
        match GilrsBuilder::new()
            .with_default_filters(false)
            .set_update_state(false)
            .build()
        {
            Ok(gilrs) => {
                app.insert_non_send_resource(gilrs)
                    .add_startup_system_to_stage(
                        StartupStage::PreStartup,
                        gilrs_event_startup_system,
                    )
                    .add_system_to_stage(
                        CoreStage::PreUpdate,
                        gilrs_event_system.before(InputSystem),
                    );
            }
            Err(err) => error!("Failed to start Gilrs. {}", err),
        }
    }
}
