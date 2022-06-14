use crate::{
    animation_pose::Pose, tmp::animation_graph_context::GraphContext,
    tmp::animation_graph_resources::GraphVariation, tmp::animation_task_system::TaskSystem,
    tmp::graph_instance::GraphInstance,
};
use lgn_transform::components::Transform;

pub struct AnimationGraphComponent {
    graph_variation: GraphVariation,
    graph_instance: GraphInstance,
    task_system: TaskSystem,
    root_motion_delta: Transform,
    graph_context: GraphContext,
    pose: Pose,
}

impl AnimationGraphComponent {
    fn get_skeleton() {
        /* */
    }

    fn get_pose() {
        /* */
    }

    #[inline]
    fn get_root_motion_delta() {
        /* */
    }

    fn evaluate_graph() {
        /* */
    }

    fn execute_pre_physics_tasks() {
        /* */
    }

    fn execute_post_physics_tasks() {
        /* */
    }
}
