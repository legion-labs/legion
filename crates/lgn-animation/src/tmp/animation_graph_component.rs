use crate::{
    animation_graph_context::GraphContext, animation_graph_resources::GraphVariation,
    animation_pose::Pose, animation_skeleton::Skeleton, animation_task_system::TaskSystem,
    graph_instance::GraphInstance,
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
