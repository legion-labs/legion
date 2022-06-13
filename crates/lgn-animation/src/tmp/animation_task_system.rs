use lgn_transform::components::Transform;

use crate::{animation_task::SampleTask, animation_task_pose_pool::PoseBufferPool};

pub enum TaskUpdateStage {
    Any = 0,
    PrePhysics,
    PostPhysics,
}
pub struct TaskContext {
    world_transform: Transform,
    world_transform_inverse: Transform,
    dependencies: Vec<SampleTask>,
    delta_time: f32,
    update_stage: TaskUpdateStage,
    pose_pool: PoseBufferPool,
}
pub struct TaskSystem {
    tasks: Vec<SampleTask>,
    pose_pool: PoseBufferPool,
    task_context: TaskContext,
    pre_physics_task_indices: Vec<i8>,
    has_physics_dependency: bool,
    has_codependent_physics_tasks: bool,
}

impl TaskSystem {
    pub fn reset() {
        /* */
    }

    pub fn get_character_world_transform() {
        /* */
    }

    pub fn update_pre_physics() {
        /* */
    }

    pub fn update_post_physics() {
        /* */
    }

    #[inline]
    pub fn has_tasks() {
        /* */
    }

    pub fn get_current_task_index_marker() {
        /* */
    }

    pub fn rollback_to_task_index_marker() {
        /* */
    }

    pub fn add_task_chain_to_pre_physics_list() {
        /* */
    }

    pub fn execute_tasks() {
        /* Calls Execute for task every task in tasks */
    }
}
