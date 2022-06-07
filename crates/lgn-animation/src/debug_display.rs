use crate::components::AnimationClip;
use crate::{animation_options::AnimationOptions, components::GraphDefinition};
use lgn_core::BumpAllocatorPool;
use lgn_ecs::prelude::{Query, Res};
use lgn_graphics_data::Color;
use lgn_graphics_renderer::{debug_display::DebugDisplay, resources::DefaultMeshType};

pub(crate) fn display_animation(
    debug_display: Res<'_, DebugDisplay>,
    bump_allocator_pool: Res<'_, BumpAllocatorPool>,
    animation_options: Res<'_, AnimationOptions>,
    animations: Query<'_, '_, &AnimationClip>,
    graphs: Query<'_, '_, &GraphDefinition>,
) {
    if !animation_options.show_collision_geometry {
        return;
    }

    bump_allocator_pool.scoped_bump(|bump| {
        debug_display.create_display_list(bump, |builder| {
            let debug_color = Color::new(0, 255, 52, 255);
            for graph in graphs.iter() {
                for n_bone in 0..graph.nodes[graph.current_node_index as usize]
                    .clip
                    .skeleton
                    .bone_ids
                    .len()
                {
                    builder.add_default_mesh(
                        &graph.nodes[graph.current_node_index as usize]
                            .clip
                            .skeleton
                            .poses[graph.nodes
                            [graph.current_node_index as usize]
                            .clip
                            .current_key_frame_index as usize][n_bone]
                            .global,
                        DefaultMeshType::Sphere,
                        debug_color,
                    );
                }
            }
            // for animation in animations.iter() {
            //     for n_bone in 0..animation.skeleton.bone_ids.len() {
            //         builder.add_default_mesh(
            //             &animation.skeleton.poses[animation.current_key_frame_index as usize]
            //                 [n_bone]
            //                 .global,
            //             DefaultMeshType::Sphere,
            //             debug_color,
            //         );
            //     }
            // }
        });
    });

    drop(debug_display);
    drop(bump_allocator_pool);
    drop(animation_options);
    drop(animations);
    drop(graphs);
}
