use lgn_transform::components::Transform;

pub struct Skeleton {
    bone_ids: Vec<u32>,
    parent_indices: Vec<i32>,
    local_reference_pose: Vec<Transform>,
    global_reference_pose: Vec<Transform>,
    // const bone_flags: Vec<BoneFlags>, TODO BoneFlags
}

impl Skeleton {
    #[inline]
    fn get_num_bones() {
        /* */
    }

    #[inline]
    fn is_valid_bone_index() {
        /* */
    }

    #[inline]
    fn get_bone_index() {
        /* */
    }

    #[inline]
    fn get_parent_bone_index() {
        /* */
    }

    fn get_first_child_bone_index(boneIdx: i32) {
        /* */
    }

    fn is_child_bone_of() {}

    #[inline]
    fn is_parent_bone_of() {
        /* */
    }

    #[inline]
    fn are_bones_in_same_hierarchy() {
        /* */
    }

    fn get_bone_global_transform(idx: i32) {
        /* */
    }
}
