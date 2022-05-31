use lgn_ecs::prelude::{Res, ResMut};

use crate::{egui::Egui, RenderScope, Renderer};

#[derive(Default)]
pub(crate) struct RendererOptions {
    pub(crate) show_bounding_spheres: bool,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn ui_renderer_options(
    egui: Res<'_, Egui>,
    mut renderer_options: ResMut<'_, RendererOptions>,
    renderer: Res<'_, Renderer>,
) {
    let render_scope = renderer.render_resources().get::<RenderScope>();

    egui.window("Renderer options", |ui| {
        ui.set_min_width(400.0);
        ui.label(&format!(
            "Renderer CPU frame time: {} ms",
            render_scope.frame_time().as_secs_f32() * 1000.0
        ));
        ui.checkbox(
            &mut renderer_options.show_bounding_spheres,
            "Show bounding spheres",
        );
    });
}
