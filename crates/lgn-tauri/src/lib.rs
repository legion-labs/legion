//! Tauri plugin for Legion's ECS.
//!
//! Provides Tauri integration into Legion's ECS.

// crate-specific lint exceptions:
//#![allow()]

use std::sync::Mutex;

use lgn_app::prelude::*;
pub use lgn_tauri_macros::*;

pub struct TauriPluginSettings<R: tauri::Runtime> {
    builder: tauri::Builder<R>,
}

impl<R: tauri::Runtime> TauriPluginSettings<R> {
    pub fn new(builder: tauri::Builder<R>) -> Self {
        Self { builder }
    }
}

/// Provides game-engine integration into Tauri's event loop.
pub struct TauriPlugin<A: tauri::Assets> {
    context: Mutex<Option<tauri::Context<A>>>,
}

impl<A: tauri::Assets> TauriPlugin<A> {
    pub fn new(context: tauri::Context<A>) -> Self {
        Self {
            context: Mutex::new(Some(context)),
        }
    }
}

impl<A: tauri::Assets> Plugin for TauriPlugin<A> {
    fn build(&self, app: &mut App) {
        let context = std::mem::replace(&mut *self.context.lock().unwrap(), None).unwrap();

        app.set_runner(move |mut app| {
            let settings = app
                .world
                .remove_non_send_resource::<TauriPluginSettings<tauri::Wry>>()
                .expect("the Tauri plugin was not configured");

            let tauri_app = settings
                .builder
                .build(context)
                .expect("failed to build Tauri application");

            tauri_app.run(move |_, event| {
                if let tauri::RunEvent::MainEventsCleared = event {
                    app.update();
                }
            });
        });
    }
}
