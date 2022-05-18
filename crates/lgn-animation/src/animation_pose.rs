#![allow(dead_code)]

use lgn_transform::components::Transform;

use crate::animation_skeleton::Skeleton;

pub enum InitialState {
    None,
    ReferencePose,
    ZeroPose,
}

pub enum State {
    Unset,
    Pose,
    ReferencePose,
    ZeroPose,
    AdditivePose,
}

pub struct Pose {
    skeleton: Skeleton,
    local_transforms: Vec<Transform>,
    global_transforms: Vec<Transform>,
    state: State,
}

impl Pose {
    #[inline]
    fn get_num_bones() {
        /* */
    }

    fn reset() {
        /* */
    }

    #[inline]
    fn is_pose_set() {
        /* */
    }

    #[inline]
    fn is_reference_pose() {
        /* */
    }

    #[inline]
    fn is_zero_pose() {
        /* */
    }

    #[inline]
    fn is_additive_pose() {
        /* */
    }

    #[inline]
    fn get_transform() {
        /* */
    }

    #[inline]
    fn set_transform() {
        /* */
    }

    #[inline]
    fn set_rotation() {
        /* */
    }

    #[inline]
    fn set_translation() {
        /* */
    }

    #[inline]
    fn set_scale() {
        /* */
    }

    fn calculate_global_transforms() {
        /* */
    }

    fn get_global_transform() {
        /* */
    }
}
