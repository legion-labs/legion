use crate::{animation_skeleton::Skeleton, tmp::animation_task_system::TaskSystem};
use lgn_transform::components::Transform;

pub struct GraphLayerContext {
    layer_weight: f32,
    is_currently_in_layer: bool,
}

impl GraphLayerContext {
    #[inline]
    fn is_set() {
        /* */
    }

    #[inline]
    fn begin_layer() {
        /* */
    }

    #[inline]
    fn end_layer() {
        /* */
    }
}

pub struct GraphContext {
    graph_user_id: u64,
    task_system: TaskSystem,
    skeleton: Skeleton,
    // previous_pose: Pose,
    delta_time: f32, // Seconds
    world_transform: Transform,
    world_transform_inverse: Transform,
    // sampled_events_buffer: SampledEventsBuffer,
    update_id: u32,
    // branch_state: BranchState,
    // physics_scene: PxScene,
    // bone_mask_pool: BoneMaskPool,
    layer_context: GraphLayerContext,
}

impl GraphContext {
    pub fn initialize() {
        /* */
    }

    pub fn shutdown() {
        /* */
    }

    #[inline]
    pub fn is_valid() {
        /* */
    }

    pub fn update() {
        /* */
    }
}
