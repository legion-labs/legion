use lgn_app::{CoreStage, Plugin};
use anyhow::Context;
use lgn_ecs::prelude::*;
use lgn_graphics_api::QueueType;
use lgn_math::{EulerRot, Quat};
use lgn_transform::components::Transform;
use lgn_egui::{Egui, EguiPlugin};

use crate::{
    components::{RenderSurface, RotationComponent, StaticMesh},
    labels::RendererSystemLabel,
    RenderContext, Renderer,
};

#[derive(Default)]
pub struct RendererPlugin;

#[derive(Default)]
struct RendererUI {
    text: String,
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut lgn_app::App) {
        let renderer = Renderer::new().unwrap();
        app.add_plugin(EguiPlugin::default());
        app.insert_resource(renderer);
        app.insert_resource(RendererUI {
            text: String::from("something"),
        });

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update);
        app.add_system_to_stage(CoreStage::PreUpdate, update_ui.system());
        // Update
        app.add_system(update_rotation.before(RendererSystemLabel::FrameUpdate));

        app.add_system_set(
            SystemSet::new()
                .with_system(render_update)
                .label(RendererSystemLabel::FrameUpdate),
        );

        // Post-Update
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            render_post_update, // .label(RendererSystemLabel::FrameDone),
        );
    }
}

fn update_ui(
    mut egui_ctx: ResMut<Egui>,
    mut renderer_ui: ResMut<RendererUI>,
    mut rotations: Query<'_, '_, &mut RotationComponent>,
) {
    egui::Window::new("Update rotation speeds").show(&egui_ctx.ctx, |ui| {
        for (i, mut rotation_component) in rotations.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Object {}: ", i));
                ui.add(
                    egui::Slider::new(&mut rotation_component.rotation_speed.0, 0.0..=5.0)
                        .text("x"),
                );
                ui.add(
                    egui::Slider::new(&mut rotation_component.rotation_speed.1, 0.0..=5.0)
                        .text("y"),
                );
                ui.add(
                    egui::Slider::new(&mut rotation_component.rotation_speed.2, 0.0..=5.0)
                        .text("z"),
                );
            });
        }
        ui.text_edit_multiline(&mut renderer_ui.text);
    });
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
    renderer: ResMut<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    q_drawables: Query<'_, '_, (&Transform, &StaticMesh)>,
    task_pool: Res<'_, crate::RenderTaskPool>,
    mut egui: ResMut<'_, Egui>,
) {
    let mut render_context = RenderContext::new(&renderer);
    let q_drawables = q_drawables
        .iter()
        .collect::<Vec<(&Transform, &StaticMesh)>>();
    let graphics_queue = renderer.queue(QueueType::Graphics);

    // For each surface/view, we have to execute the render graph
    for mut render_surface in q_render_surfaces.iter_mut() {
        // TODO: render graph
        let cmd_buffer = render_context.acquire_cmd_buffer(QueueType::Graphics);

        cmd_buffer.begin().unwrap();

        let render_pass = render_surface.test_renderpass();
        let render_pass = render_pass.write();
        render_pass.render(
            &mut render_context,
            &cmd_buffer,
            render_surface.as_mut(),
            q_drawables.as_slice(),
        );
        cmd_buffer.end().unwrap();
        // queue
        let sem = render_surface.acquire();
        graphics_queue
            .submit(&[&cmd_buffer], &[], &[sem], None)
            .unwrap();

        render_context.release_cmd_buffer(cmd_buffer);

        render_surface.present(&mut render_context, &task_pool);
    }
}

fn render_post_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.end_frame();
}
