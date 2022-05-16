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

use lgn_app::{App, Plugin};

#[derive(Default)]
pub struct AnimationPlugin {}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        /* */
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
