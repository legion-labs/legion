//! Async plugin for Legion's ECS.
//!
//! Provides an async <-> sync bridge between Legion's ECS systems and various
//! async runtimes, like `tokio`.

// crate-specific lint exceptions:
//#![allow()]

use lgn_app::prelude::*;
use lgn_ecs::prelude::*;

pub mod operation;
pub mod runtime;
pub mod sync;

pub use operation::*;
pub use runtime::*;

// Provides async online capabilities via an online runtime.
#[derive(Default)]
pub struct AsyncPlugin;

impl Plugin for AsyncPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TokioAsyncRuntime::default());
        app.add_system(|mut rt: ResMut<'_, TokioAsyncRuntime>| {
            rt.poll();
        });
    }
}
