// generated from def\animation.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod animation_clip;
mod animation_event;
mod animation_frame_time;
mod animation_graph_component;
mod animation_graph_context;
mod animation_graph_events;
mod animation_graph_resources;
mod animation_pose;
mod animation_skeleton;
mod animation_sync_track;
mod animation_system;
mod animation_task;
mod animation_task_pose_pool;
mod animation_task_system;
mod graph_instance;
mod graph_nodes;
mod labels;

use crate::{animation_system::update, labels::AnimationStage};
use lgn_app::{App, CoreStage, Plugin};
use lgn_ecs::schedule::SystemStage;

#[derive(Default)]
pub struct AnimationPlugin {}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        // app.add_stage(AnimationStage::Update, SystemStage::parallel());
        app.add_stage_before(
            CoreStage::Update,
            AnimationStage::Update,
            SystemStage::parallel(),
        );
        app.add_system_to_stage(AnimationStage::Update, update);
    }
}

impl AnimationPlugin {}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
