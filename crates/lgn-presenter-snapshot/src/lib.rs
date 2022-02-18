//! Presenter plugin made for windowing system.

// crate-specific lint exceptions:
//#![allow()]

mod cgen {
    include!(concat!(env!("OUT_DIR"), "/rust/mod.rs"));
}
use std::sync::Arc;

#[allow(unused_imports)]
use cgen::*;

pub mod component;

mod offscreen_capture;

use lgn_app::{App, Plugin};
use lgn_ecs::prelude::ResMut;
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_renderer::Renderer;
use offscreen_capture::OffscreenHelper;

#[derive(Default)]
pub struct PresenterSnapshotPlugin;

impl Plugin for PresenterSnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_cgen);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_cgen(
    mut renderer: ResMut<'_, Renderer>,
    mut cgen_registries: ResMut<'_, CGenRegistryList>,
) {
    let cgen_registry = Arc::new(cgen::initialize(renderer.device_context()));
    renderer
        .pipeline_manager_mut()
        .register_shader_families(&cgen_registry);
    cgen_registries.push(cgen_registry);
}
