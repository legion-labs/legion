#![allow(dead_code)]

use crate::{animation_clip::AnimationClip, animation_task_system::TaskUpdateStage};

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
    const SOURCE_ID: i16;
    const UPDATE_STAGE: TaskUpdateStage;
    const BUFFER_IDX: i8;
    const DEPENDENCIES: Vec<i8>;
    const IS_COMPLETE: bool;
}

pub struct _SampleTask {
    animation_clip: AnimationClip,
    time: u32,
}

// impl Task for SampleTask {
//     /* */
// }
