use crate::animation_skeleton::Skeleton;

pub struct BoneMask {
    skeleton: Skeleton,
    weights: Vec<f32>,
    root_motion_weight: f32,
}

// impl BoneMask {
//     pub fn new(skeleton: &Skeleton) -> Self {
//         Self {
//             skeleton: todo!(),
//             weights: todo!(),
//             root_motion_weight: todo!(),
//         }
//     }

//     #[inline]
//     pub fn is_valid() {}

//     // Set all weights to the supplied weights
//     pub fn reset_weights() {}

//     // Multiple te supplied bone mask into the current bone mask
//     #[inline]
//     pub fn combine_with() {}

//     // Blend from the supplied mask weight towards our weights with the supplied blend weight
//     pub fn blend_from() {}

//     // Blend towards the supplied mask weights from our current weights with the supplied blend weight
//     pub fn blend_to() {}
// }
