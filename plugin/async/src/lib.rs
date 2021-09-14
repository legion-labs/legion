//! Async plugin for Legion's ECS.
//!
//! Provides an async <-> sync bridge between Legion's ECS systems and various
//! async runtimes, like `tokio`.

use legion_app::prelude::{App, Plugin};
use legion_ecs::prelude::ResMut;

pub mod operation;
pub mod runtime;

pub use operation::*;
pub use runtime::*;

// Provides async online capabilities via an online runtime.
pub struct AsyncPlugin {}

impl Plugin for AsyncPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TokioAsyncRuntime::default());
        app.add_system(|mut rt: ResMut<TokioAsyncRuntime>| {
            rt.poll();
        });
    }
}
