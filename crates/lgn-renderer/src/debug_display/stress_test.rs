use lgn_app::{App, CoreStage};
use lgn_core::BumpAllocatorPool;
use lgn_ecs::prelude::Res;
use lgn_math::Vec3;
use lgn_tracing::span_fn;
use lgn_transform::components::Transform;

use crate::resources::DefaultMeshType;

use super::DebugDisplay;

// To test, call this function in build() of the plugin
pub fn debug_stress_test(app: &mut App) {
    for _i in 1..100 {
        app.add_system_to_stage(CoreStage::Update, add_debug_things);
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
pub fn add_debug_things(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
) {
    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            for _i in 1..1000 {
                builder.add_mesh(
                    Transform::identity().compute_matrix(),
                    DefaultMeshType::Sphere as u32,
                    Vec3::ZERO,
                );
            }
        });
    });
}
