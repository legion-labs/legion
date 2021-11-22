use legion_app::{CoreStage, Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};
use legion_math::{EulerRot, Quat};
use legion_transform::components::Transform;

use crate::{
    components::{RenderSurface, RotationComponent, StaticMesh},
    labels::RendererSystemLabel,
    Renderer,
};

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new().unwrap();

        app.insert_resource(renderer);

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update.system());
        app.add_system_to_stage(CoreStage::PreUpdate, update_rotation.system());

        // Update
        app.add_system_set(
            SystemSet::new()
                .with_system(render_update.system())
                .label(RendererSystemLabel::FrameUpdate),
        );

        // Post-Update
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            render_post_update
                .system()
                .label(RendererSystemLabel::FrameDone),
        );
    }
}

fn render_pre_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.begin_frame();
}

fn update_rotation(mut query: Query<'_, '_, (&mut Transform, &RotationComponent)>) {
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotate(Quat::from_euler(
            EulerRot::XYZ,
            rotation.rotation_speed.0 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.1 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.2 / 60.0 * std::f32::consts::PI,
        ));
    }
}

#[allow(clippy::needless_pass_by_value)]
fn render_update(
    mut renderer: ResMut<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    query: Query<'_, '_, (&Transform, &StaticMesh)>,
) {
    renderer.update(&mut q_render_surfaces, &query);
}

fn render_post_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.end_frame();
}
