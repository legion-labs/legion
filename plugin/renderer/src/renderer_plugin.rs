use crate::{
    components::PickedComponent,
    egui::egui_plugin::{Egui, EguiPlugin},
    picking::{PickingManager, PickingPlugin},
};
use lgn_app::{CoreStage, Plugin};
use lgn_ecs::prelude::*;
use lgn_graphics_api::QueueType;
use lgn_math::{EulerRot, Quat};
use lgn_transform::components::Transform;

use crate::{
    components::{RenderSurface, RotationComponent, StaticMesh},
    labels::RendererSystemLabel,
    resources::{EntityTransforms, UniformGPUDataUpdater},
    RenderContext, Renderer,
};

#[derive(Default)]
pub struct RendererPlugin {
    has_window: bool,
    enable_egui: bool,
}

impl RendererPlugin {
    pub fn new(has_window: bool, enable_egui: bool) -> Self {
        Self {
            has_window,
            enable_egui,
        }
    }
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut lgn_app::App) {
        let renderer = Renderer::new().unwrap();
        app.add_plugin(EguiPlugin::new(self.has_window, self.enable_egui));
        app.add_plugin(PickingPlugin::new(self.has_window));
        app.insert_resource(renderer);

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update);
        // Update
        app.add_system(update_rotation.before(RendererSystemLabel::FrameUpdate));
        app.add_system(update_ui.before(RendererSystemLabel::FrameUpdate));

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

#[allow(clippy::needless_pass_by_value)]
fn update_ui(egui_ctx: Res<'_, Egui>, mut rotations: Query<'_, '_, &mut RotationComponent>) {
    egui::Window::new("Rotations").show(&egui_ctx.ctx, |ui| {
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
    });
}

fn render_pre_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.begin_frame();
}

fn update_rotation(
    mut renderer: ResMut<'_, Renderer>,
    mut query: Query<'_, '_, (Entity, &mut Transform, &RotationComponent, &mut StaticMesh)>,
) {
    let mut updater = UniformGPUDataUpdater::new(renderer.transient_buffer(), 64 * 1024);
    let mut gpu_data = renderer.aquire_transform_data();

    for (entity, mut transform, rotation, mut mesh) in query.iter_mut() {
        mesh.offset = gpu_data.ensure_index_allocated(entity.id());

        transform.rotate(Quat::from_euler(
            EulerRot::XYZ,
            rotation.rotation_speed.0 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.1 / 60.0 * std::f32::consts::PI,
            rotation.rotation_speed.2 / 60.0 * std::f32::consts::PI,
        ));

        let world = EntityTransforms {
            world: transform.compute_matrix(),
        };

        updater.add_update_jobs(&[world], mesh.offset);
    }

    renderer.test_add_update_jobs(updater.job_blocks());

    renderer.release_transform_data(gpu_data);
}

#[allow(clippy::needless_pass_by_value)]
fn render_update(
    renderer: ResMut<'_, Renderer>,
    picking_manager: ResMut<'_, PickingManager>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    q_drawables: Query<'_, '_, (&StaticMesh, Option<&PickedComponent>)>,
    task_pool: Res<'_, crate::RenderTaskPool>,
    mut egui: ResMut<'_, Egui>,
) {
    crate::egui::egui_plugin::end_frame(&mut egui);

    let mut render_context = RenderContext::new(&renderer);
    let q_drawables = q_drawables
        .iter()
        .collect::<Vec<(&StaticMesh, Option<&PickedComponent>)>>();
    let graphics_queue = renderer.queue(QueueType::Graphics);

    renderer.flush_update_jobs(&mut render_context, &graphics_queue);

    // For each surface/view, we have to execute the render graph
    for mut render_surface in q_render_surfaces.iter_mut() {
        let picking_pass = render_surface.picking_renderpass();
        let mut picking_pass = picking_pass.write();
        picking_pass.render(
            &picking_manager,
            &mut render_context,
            render_surface.as_mut(),
            q_drawables.as_slice(),
        );

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

        let egui_pass = render_surface.egui_renderpass();
        let mut egui_pass = egui_pass.write();
        egui_pass.update_font_texture(&mut render_context, &cmd_buffer, &egui.ctx);
        if egui.enable {
            egui_pass.render(
                &mut render_context,
                &cmd_buffer,
                render_surface.as_mut(),
                &egui,
            );
        }

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
