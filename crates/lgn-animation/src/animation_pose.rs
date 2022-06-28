use lgn_math::{Quat, Vec3};
use lgn_transform::{
    prelude::{GlobalTransform, Transform},
    TransformBundle,
};
use std::sync::Arc;

use crate::animation_skeleton::Skeleton;

#[derive(Clone)]
pub struct Pose {
    pub(crate) skeleton: Arc<Skeleton>,
    pub(crate) transforms: Vec<TransformBundle>,
    pub(crate) root_motion: GlobalTransform,
    pub(crate) current_root_position: GlobalTransform,
}

impl Pose {
    #[inline]
    pub(crate) fn get_num_bones(&self) -> usize {
        self.skeleton.bone_ids.len()
    }

    #[inline]
    pub(crate) fn get_bone_transform(&self, bone_idx: usize) -> GlobalTransform {
        self.transforms[bone_idx].global
    }

    #[inline]
    pub(crate) fn set_local_transform(&mut self, bone_idx: usize, bone_transform: Transform) {
        self.transforms[bone_idx].local = bone_transform;
    }

    #[inline]
    fn set_rotation(&mut self, bone_idx: usize, rotation: Quat) {
        self.transforms[bone_idx].local.rotation = rotation;
    }

    #[inline]
    fn set_translation(&mut self, bone_idx: usize, translation: Vec3) {
        self.transforms[bone_idx].local.translation = translation;
    }

    #[inline]
    fn set_scale(&mut self, bone_idx: usize, scale: Vec3) {
        self.transforms[bone_idx].local.scale = scale;
    }

    pub(crate) fn calculate_global_transforms(&mut self) {
        for n_bone in 0..self.skeleton.bone_ids.len() {
            if !self.is_root_bone(n_bone) {
                self.transforms[n_bone].global = self.transforms
                    [self.skeleton.parent_indices[n_bone].unwrap()]
                .global
                .mul_transform(self.transforms[n_bone].local);
            } else {
                self.transforms[n_bone].global = self.transforms[n_bone].local.into();
            }
        }
    }

    // fn apply_root_motion(&mut self) {
    //     for n_bone in 0..self.skeleton.bone_ids.len() {
    //         self.transforms[n_bone].global = self.transforms[n_bone]
    //             .global
    //             .add(self.root_motion.add(self.current_root_position));
    //     }
    // }

    fn is_root_bone(&self, bone_index: usize) -> bool {
        self.skeleton.parent_indices[bone_index].is_none()
    }
}
