use lgn_app::{App, CoreStage};
use lgn_ecs::prelude::Res;
use lgn_graphics_data::Color;
use lgn_tracing::span_fn;
use lgn_transform::components::GlobalTransform;

use crate::{
    debug_display::{DebugPrimitiveMaterial, DebugPrimitiveType},
    resources::DefaultMeshType,
};

use super::DebugDisplay;

// To test, call this function in build() of the plugin
pub fn debug_stress_test(app: &mut App) {
    for _i in 1..100 {
        app.add_system_to_stage(CoreStage::Update, add_debug_things);
    }
}

#[span_fn]
#[allow(clippy::needless_pass_by_value)]
pub fn add_debug_things(debug_display: Res<'_, DebugDisplay>) {
    debug_display.create_display_list(|builder| {
        for _i in 1..1000 {
            builder.add_draw_call(
                &GlobalTransform::identity(),
                DebugPrimitiveType::default_mesh(DefaultMeshType::Sphere),
                Color::BLACK,
                DebugPrimitiveMaterial::WireDepth,
            );
        }
    });
}
