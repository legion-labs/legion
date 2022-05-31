#![allow(dead_code)]

use crate::{
    animation_graph_context::GraphContext, animation_graph_resources::GraphVariation,
    animation_pose::Pose, animation_task_system::TaskSystem, graph_instance::GraphInstance,
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
    fn initialize(&self) {
        // EntityComponent::Initialize();

        // Create new Pose with skeleton
        // Calc transforms? Si je recois du raw data (ou bien je pourrais le faire avant, comme ça on initialise au début une seule fois!)

        // Create a new task system -> pense pas que c'est necessaire pour l'instant
        // Creer une instance de graph
        // self.graph_instance = GraphInstance::new()
    }

    fn get_skeleton() {}

    fn get_pose() {}

    #[inline]
    fn get_root_motion_delta() {}

    fn evaluate_graph() {}

    fn execute_pre_physics_tasks() {}

    fn execute_post_physics_tasks() {}
}
