use legion_app::prelude::{App, Plugin};

pub mod operation;
pub mod runtime;

pub use operation::*;
pub use runtime::*;

// Provides async online capabilities via an online runtime.
pub struct AsyncPlugin;

impl Plugin for AsyncPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TokioAsyncRuntime::default());
    }
}
