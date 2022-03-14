use lgn_app::{App, EventReader, Events, Plugin};
use lgn_ecs::prelude::{IntoExclusiveSystem, Query, Res, ResMut, Without};
use lgn_transform::components::GlobalTransform;

use crate::{
    components::{CameraComponent, LightComponent, ManipulatorComponent, VisualComponent},
    egui::egui_plugin::Egui,
};

pub(crate) struct SceneExporterPlugin {}

impl Plugin for SceneExporterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ui_scene_exporter);
        app.add_system(on_scene_export_requested.exclusive_system());
        app.add_event::<SceneExportRequested>();
    }
}

pub(crate) fn on_scene_export_requested(
    mut event_scene_export_requested: EventReader<'_, '_, SceneExportRequested>,
    visuals: Query<'_, '_, (&VisualComponent, &GlobalTransform), Without<ManipulatorComponent>>,
    lights: Query<'_, '_, (&LightComponent, &GlobalTransform)>,
    cameras: Query<'_, '_, (&CameraComponent, &GlobalTransform)>,
) {
    if !event_scene_export_requested.is_empty() {
        unimplemented!();
    }
}

pub(crate) fn ui_scene_exporter(
    egui: Res<'_, Egui>,
    mut event_scene_export_requested: ResMut<'_, Events<SceneExportRequested>>,
) {
    egui::Window::new("Scene export").show(&egui.ctx, |ui| {
        if ui.button("Export").clicked() {
            event_scene_export_requested.send(SceneExportRequested {});
        }
    });
}

pub(crate) struct SceneExportRequested {}
