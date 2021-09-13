use legion_app::prelude::{App, Plugin};

pub mod runtime;

pub use runtime::*;

// Provides async online capabilities via an online runtime.
pub struct OnlinePlugin;

impl Plugin for OnlinePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Runtime::default());
    }
}
