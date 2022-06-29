use crate::{
    animation_options::AnimationOptions, components::GraphDefinition,
    runtime_graph::node_state_machine::StateInfo,
};
use lgn_ecs::prelude::{Query, Res};
use lgn_graphics_data::Color;
use lgn_graphics_renderer::{
    debug_display::{DebugDisplay, DebugPrimitiveMaterial, DebugPrimitiveType},
    resources::DefaultMeshType,
};

pub(crate) fn display_animation(
    debug_display: Res<'_, DebugDisplay>,
    animation_options: Res<'_, AnimationOptions>,
    graphs: Query<'_, '_, &GraphDefinition>,
) {
    if !animation_options.show_animation_skeleton_bones {
        return;
    }

    debug_display.create_display_list(|builder| {
        for graph in graphs.iter() {
            let active_state: &StateInfo = graph.get_current_node().get_active_state().unwrap();
            let clip = active_state.state_node.child_node.get_clip().unwrap();

            for n_bone in 0..clip.poses[clip.current_key_frame_index]
                .skeleton
                .bone_ids
                .len()
            {
                let bone_depth: u8 = clip.poses[clip.current_key_frame_index]
                    .skeleton
                    .get_bone_depth(
                        clip.poses[clip.current_key_frame_index].skeleton.bone_ids[n_bone].unwrap(),
                    )
                    .try_into()
                    .unwrap();
                let color_interval: u8 = (255
                    / clip.poses[clip.current_key_frame_index]
                        .skeleton
                        .get_max_bone_depth()
                        .unwrap())
                .try_into()
                .unwrap();
                let debug_color = Color::new(bone_depth * color_interval, 255, 52, 255);
                builder.add_default_mesh(
                    &clip.poses[clip.current_key_frame_index].transforms[n_bone].global,
                    DebugPrimitiveType::default_mesh(DefaultMeshType::Sphere),
                    debug_color,
                    DebugPrimitiveMaterial::WireDepth,
                );
            }
        }
    });
    drop(debug_display);
    drop(animation_options);
    drop(graphs);
}
