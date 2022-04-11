use lgn_ecs::prelude::{Res, ResMut};

use crate::egui::Egui;

#[derive(Default)]
pub(crate) struct RendererOptions {
    pub(crate) show_bounding_spheres: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_renderer_options(
    egui: Res<'_, Egui>,
    mut renderer_options: ResMut<'_, RendererOptions>,
) {
    egui.window("Renderer options", |ui| {
        ui.checkbox(
            &mut renderer_options.show_bounding_spheres,
            "Show bounding spheres",
        );
    });
}
