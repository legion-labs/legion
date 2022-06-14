use crate::animation_pose::Pose;
use crate::animation_skeleton::Skeleton;
use crate::runtime::{AnimationTrack, AnimationTransformBundleVec};

use lgn_ecs::component::Component;
use lgn_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};

#[derive(Component, Clone)]
pub struct AnimationClip {
    pub(crate) current_key_frame_index: u32,
    pub(crate) duration_key_frames: Vec<f32>,
    pub(crate) time_since_last_tick: f32,
    pub(crate) looping: bool,
    pub(crate) poses: Vec<Pose>,
}

impl AnimationClip {
    #[must_use]
    pub fn new(raw_animation_track: &AnimationTrack) -> Self {
        let mut bone_ids: Vec<Option<usize>> = Vec::new();
        raw_animation_track
            .bone_ids
            .iter()
            .for_each(|bone_id| bone_ids.push(Some((*bone_id).try_into().unwrap())));
        let mut parent_indices: Vec<Option<usize>> = Vec::new();
        raw_animation_track
            .parent_indices
            .iter()
            .for_each(|parent_idx| {
                if *parent_idx >= 0 {
                    parent_indices.push(Some((*parent_idx).try_into().unwrap()));
                } else {
                    parent_indices.push(None);
                }
            });
        let skeleton = Skeleton {
            bone_ids,
            parent_indices,
        };
        let converted_transforms = convert_raw_pose_data(&raw_animation_track.key_frames);
        let mut converted_poses = Vec::new();

        for pose_transforms in converted_transforms {
            converted_poses.push(Pose {
                skeleton: skeleton.clone(),
                transforms: pose_transforms,
            });
        }

        update_children_transforms(&mut converted_poses);

        Self {
            current_key_frame_index: raw_animation_track.current_key_frame_index,
            duration_key_frames: raw_animation_track.duration_key_frames.clone(),
            time_since_last_tick: raw_animation_track.time_since_last_tick,
            looping: raw_animation_track.looping,
            poses: converted_poses,
        }
    }
}

fn convert_raw_pose_data(
    raw_poses: &Vec<AnimationTransformBundleVec>,
) -> Vec<Vec<TransformBundle>> {
    let mut poses: Vec<Vec<TransformBundle>> = Vec::new();
    for pose in raw_poses {
        let mut vec_transform_bundle = Vec::new();
        for anim_transform_bundle in &pose.anim_transform_vec {
            vec_transform_bundle.push(TransformBundle {
                local: Transform {
                    translation: anim_transform_bundle.translation,
                    rotation: anim_transform_bundle.rotation,
                    scale: anim_transform_bundle.scale,
                },
                global: GlobalTransform::identity(),
            });
        }
        poses.push(vec_transform_bundle);
    }
    poses
}

fn update_children_transforms(poses: &mut Vec<Pose>) {
    for pose in poses {
        pose.calculate_global_transforms();
    }
}
