use crate::components::GraphDefinition;
use lgn_ecs::prelude::{Query, Res, ResMut};
use lgn_graphics_renderer::egui::Egui;

#[derive(Default)]
pub(crate) struct AnimationOptions {
    pub(crate) show_animation_skeleton_bones: bool,
}

pub(crate) fn ui_animation_options(
    egui: Res<'_, Egui>,
    mut animation_options: ResMut<'_, AnimationOptions>,
    graphs: Query<'_, '_, &GraphDefinition>,
) {
    egui.window("Animation options", |ui| {
        ui.checkbox(
            &mut animation_options.show_animation_skeleton_bones,
            "Show animation skeleton bones",
        );
        for graph in graphs.iter() {
            ui.label(format!(
                "Current state: {}",
                graph
                    .get_current_node()
                    .get_active_state()
                    .unwrap()
                    .state_node
                    .child_node
                    .get_state_name()
                    .unwrap()
            ));
        }
    });

    drop(egui);
    drop(graphs);
}
