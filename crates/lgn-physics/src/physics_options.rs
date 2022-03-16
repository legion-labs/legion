use lgn_ecs::prelude::{Res, ResMut};

use lgn_graphics_renderer::egui::Egui;

#[derive(Default)]
pub(crate) struct PhysicsOptions {
    pub(crate) show_collision_geometry: bool,
}

pub(crate) fn ui_physics_options(
    egui: Res<'_, Egui>,
    mut physics_options: ResMut<'_, PhysicsOptions>,
) {
    egui::Window::new("Physics options").show(&egui.ctx, |ui| {
        ui.checkbox(
            &mut physics_options.show_collision_geometry,
            "Show collision geometry",
        );
    });

    drop(egui);
}
