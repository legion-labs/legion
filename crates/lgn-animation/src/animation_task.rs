use crate::{
    animation_clip::AnimationClip, animation_task_pose_pool::PoseBuffer,
    animation_task_system::TaskUpdateStage,
};

pub trait Task {
    fn execute();

    #[inline]
    fn get_num_dependencies() {
        /* */
    }

    #[inline]
    fn get_new_pose_buffer() {
        /* */
    }

    #[inline]
    fn release_pose_buffer() {
        /* */
    }

    #[inline]
    fn transfer_dependency_pose_buffer() {
        /* */
    }

    #[inline]
    fn access_dependency_pose_buffer() {
        /* */
    }

    #[inline]
    fn get_temporary_pose_buffer() {
        /* */
    }

    #[inline]
    fn mark_task_complete() {
        /* */
    }
    const source_id: i16;
    const update_stage: TaskUpdateStage;
    const buffer_idx: i8;
    const dependencies: Vec<i8>;
    const is_complete: bool;
}

pub struct SampleTask {
    animation_clip: AnimationClip,
    time: u32,
}

// impl Task for SampleTask {
//     /* */
// }
