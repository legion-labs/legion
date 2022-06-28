use crate::animation_skeleton::Skeleton;

pub struct PoseBuffer {
    is_used: bool,
}
pub struct PoseBufferPool {
    skeleton: Skeleton,
    pose_buffers: Vec<PoseBuffer>,
}

impl PoseBufferPool {
    pub fn reset() {
        /* */
    }

    pub fn request_pose_buffer() {
        /* */
    }

    pub fn release_pose_buffer() {
        /* */
    }

    pub fn get_buffer() {
        /* */
    }
}
