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
use lgn_app::{App, Plugin};
use lgn_ecs::prelude::{Res, ResMut};
use lgn_renderer::{CGenRegistries, Renderer};
use offscreen_capture::OffscreenHelper;

#[derive(Default)]
pub struct PresenterSnapshotPlugin;

impl Plugin for PresenterSnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_cgen);
    }
}

#[allow(clippy::needless_pass_by_value)]
fn init_cgen(renderer: Res<'_, Renderer>, mut cgen_registries: ResMut<'_, CGenRegistries>) {
    let cgen_registry = cgen::initialize(renderer.device_context());
    cgen_registries.push(cgen_registry);
}
