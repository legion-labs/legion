#[derive(Clone)]
pub struct Skeleton {
    pub bone_ids: Vec<i32>,
    pub parent_indices: Vec<Option<usize>>,
}

impl Skeleton {
    #[inline]
    pub fn get_num_bones(&self) -> usize {
        self.bone_ids.len()
    }

    #[inline]
    pub fn is_valid_bone_index(&self, idx: &Option<usize>) -> bool {
        idx.unwrap() < self.bone_ids.len()
    }

    #[inline]
    pub fn get_bone_index(&self, id: i32) -> usize {
        self.bone_ids
            .binary_search(&id)
            .expect("Bone is not present in skeleton.")
    }

    #[inline]
    pub fn get_parent_bone_idx(&self, bone_idx: usize) -> Option<usize> {
        self.parent_indices[bone_idx]
    }

    fn get_first_child_bone_index(&self, bone_idx: &Option<usize>) -> Option<usize> {
        assert!(self.is_valid_bone_index(bone_idx));

        self.parent_indices
            .iter()
            .enumerate()
            .find_map(|(i, parent_idx)| {
                if parent_idx == bone_idx {
                    Some(i)
                } else {
                    None
                }
            })
    }

    fn is_child_bone_of(
        &self,
        parent_bone_idx: &Option<usize>,
        child_bone_idx: &Option<usize>,
    ) -> bool {
        assert!(self.is_valid_bone_index(parent_bone_idx));
        assert!(self.is_valid_bone_index(child_bone_idx));

        let mut is_child = false;
        let mut actual_parent_bone_idx = self.parent_indices[child_bone_idx.unwrap()];

        while actual_parent_bone_idx.is_some() {
            if actual_parent_bone_idx == *parent_bone_idx {
                is_child = true;
                break;
            }
            actual_parent_bone_idx = self.parent_indices[actual_parent_bone_idx.unwrap()];
        }
        is_child
    }

    #[inline]
    fn is_parent_bone_of(
        &self,
        parent_bone_idx: &Option<usize>,
        child_bone_idx: &Option<usize>,
    ) -> bool {
        self.is_child_bone_of(parent_bone_idx, child_bone_idx)
    }

    #[inline]
    fn are_bones_in_same_hierarchy(
        &self,
        bone_idx_0: &Option<usize>,
        bone_idx_1: &Option<usize>,
    ) -> bool {
        self.is_child_bone_of(bone_idx_0, bone_idx_1)
            || self.is_child_bone_of(bone_idx_1, bone_idx_0)
    }
}
