#![allow(dead_code)]

use lgn_transform::TransformBundle;

#[derive(Clone)]
pub struct Skeleton {
    pub bone_ids: Vec<i32>,
    pub parent_indices: Vec<i32>,
    pub poses: Vec<Vec<TransformBundle>>,
    // pub local_reference_pose: Vec<Transform>,
    // pub global_reference_pose: Vec<Transform>,
    // const bone_flags: Vec<BoneFlags>, TODO BoneFlags
}

impl Skeleton {
    /* Bone info */
    #[inline]
    pub fn get_num_bones(&self) -> usize {
        self.bone_ids.len()
    }

    #[inline]
    pub fn is_valid_bone_index(&self, idx: i32) -> bool {
        idx >= 0 && idx < self.bone_ids.len() as i32
    }

    #[inline]
    pub fn get_bone_index(&self, id: &i32) -> usize {
        self.bone_ids
            .binary_search(id)
            .expect("Bone is not present in skeleton.")
    }

    #[inline]
    pub fn get_parent_bone_idx(&self, idx: i32) -> i32 {
        assert!(idx >= 0 && idx < self.parent_indices.len() as i32);

        self.parent_indices[idx as usize]
    }

    fn get_first_child_bone_index(&self, bone_idx: i32) -> i32 {
        assert!(self.is_valid_bone_index(bone_idx));

        let mut child_idx: i32 = -1; // Create macro invalid_index
        for i in bone_idx + 1..self.get_num_bones() as i32 {
            if self.parent_indices[i as usize] == bone_idx {
                child_idx = i;
                break;
            }
        }
        child_idx
    }

    fn is_child_bone_of(&self, parent_bone_idx: i32, child_bone_idx: i32) -> bool {
        assert!(self.is_valid_bone_index(parent_bone_idx));
        assert!(self.is_valid_bone_index(child_bone_idx));

        let mut is_child = false;

        let mut actual_parent_bone_idx = self.parent_indices[child_bone_idx as usize];
        while actual_parent_bone_idx != -1 {
            if actual_parent_bone_idx == parent_bone_idx {
                is_child = true;
                break;
            }
            actual_parent_bone_idx = self.parent_indices[actual_parent_bone_idx as usize];
        }

        is_child
    }

    #[inline]
    fn is_parent_bone_of(&self, parent_bone_idx: i32, child_bone_idx: i32) -> bool {
        self.is_child_bone_of(parent_bone_idx, child_bone_idx)
    }

    #[inline]
    fn are_bones_in_same_hierarchy(&self, bone_idx_0: i32, bone_idx_1: i32) -> bool {
        self.is_child_bone_of(bone_idx_0, bone_idx_1)
            || self.is_child_bone_of(bone_idx_1, bone_idx_0)
    }

    /* Pose info */
    // !TODO: change methods to get the poses and not the local_reference
    // #[inline]
    // fn get_bone_transform(&self, idx: i32) -> Transform {
    //     assert!(idx >= 0 && idx < self.local_reference_pose.len() as i32);

    //     self.local_reference_pose[idx as usize]
    // }

    // fn get_bone_global_transform(&self, idx: i32) {
    //     assert!(idx >= 0 && idx < self.local_reference_pose.len() as i32);

    //     let mut bone_global_transform = self.local_reference_pose[idx as usize];
    //     let mut parent_idx = self.get_parent_bone_idx(idx);

    //     while parent_idx != -1 {
    //         bone_global_transform =
    //             bone_global_transform * self.local_reference_pose[parent_idx as usize];
    //         parent_idx = self.get_parent_bone_idx(parent_idx);
    //     }
    // }
}
