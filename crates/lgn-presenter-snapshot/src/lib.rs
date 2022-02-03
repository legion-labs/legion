//! Presenter plugin made for windowing system.

// crate-specific lint exceptions:
//#![allow()]

mod cgen {
    include!(concat!(env!("OUT_DIR"), "/rust/mod.rs"));
}
#[allow(unused_imports)]
use cgen::*;

pub mod component;

mod offscreen_capture;
mod tmp_shader_data;

use lgn_app::{App, Plugin};
use lgn_ecs::prelude::{Res, ResMut};
use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_renderer::{resources::ShaderManager, Renderer};
use offscreen_capture::OffscreenHelper;
use tmp_shader_data::patch_cgen_registry;

#[derive(Default)]
pub struct PresenterSnapshotPlugin;

impl Plugin for PresenterSnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_cgen);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_cgen(
    renderer: Res<'_, Renderer>,
    mut shader_manager: ResMut<'_, ShaderManager>,
    mut cgen_registries: ResMut<'_, CGenRegistryList>,
) {
    let mut cgen_registry = cgen::initialize(renderer.device_context());
    patch_cgen_registry(&mut cgen_registry);
    shader_manager.register_cgen_registry(&cgen_registry);
    cgen_registries.push(cgen_registry);
}
