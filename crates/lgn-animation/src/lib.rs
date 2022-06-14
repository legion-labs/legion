//! Animation plugin for Legion's ECS

// crate-specific lint exceptions:
#![allow(dead_code)]
#![allow(clippy::cast_possible_wrap)]

// generated from def\animation.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod animation_options;
mod animation_pose;
mod animation_skeleton;
mod animation_system;
pub mod components;
mod debug_display;
mod labels;
pub mod runtime_graph;
mod tmp;

use crate::{
    animation_options::AnimationOptions, animation_system::clip_update,
    animation_system::graph_update, debug_display::display_animation,
    debug_display::display_animation_2, labels::AnimationStage,
};
use lgn_app::{App, CoreStage, Plugin};
use lgn_ecs::schedule::SystemStage;
use lgn_graphics_renderer::labels::RenderStage;

#[derive(Default)]
pub struct AnimationPlugin {}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::Update,
            AnimationStage::Update,
            SystemStage::parallel(),
        );

        // Run animation clip
        app.add_system_to_stage(AnimationStage::Update, graph_update);
        // .add_system_to_stage(AnimationStage::Update, clip_update.after(graph_update));

        // Debug display
        app.init_resource::<AnimationOptions>()
            .add_system_to_stage(
                RenderStage::Prepare,
                animation_options::ui_animation_options,
            )
            .add_system_to_stage(RenderStage::Prepare, display_animation);
    }
}
