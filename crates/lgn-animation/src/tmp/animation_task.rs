use crate::tmp::animation_clip::AnimationClip;

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
}

pub struct SampleTask {
    animation_clip: AnimationClip,
    time: u32,
}

// impl Task for SampleTask {
//     /* */
// }
