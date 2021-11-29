use graphics_api::QueueType;
use legion_app::{CoreStage, Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};
use legion_math::{EulerRot, Quat};
use legion_transform::components::Transform;

use crate::{
    components::{RenderSurface, RotationComponent, StaticMesh},
    labels::RendererSystemLabel,
    RenderContext, Renderer,
};

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new().unwrap();

        app.insert_resource(renderer);

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update.system());
        // Update
        app.add_system(
            update_rotation
                .system()
                .before(RendererSystemLabel::FrameUpdate),
        );

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

fn render_update(
    renderer: ResMut<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    q_drawables: Query<'_, '_, (&Transform, &StaticMesh)>,
) {
    let mut render_context = RenderContext::new(&renderer);
    let q_drawables = q_drawables
        .iter()
        .collect::<Vec<(&Transform, &StaticMesh)>>();
    let graphics_queue = renderer.queue(QueueType::Graphics);

    // For each surface/view, we have to execute the render graph
    for mut render_surface in q_render_surfaces.iter_mut() {
        // alloc command buffer
        let cmd_buffer = render_context.acquire_cmd_buffer(QueueType::Graphics);

        cmd_buffer.begin().unwrap();
        // flush render graph
        let render_pass = render_surface.test_renderpass();
        render_pass.render(
            &mut render_context,
            &cmd_buffer,
            render_surface.as_mut(),
            q_drawables.as_slice(),
        );
        cmd_buffer.end().unwrap();
        // queue
        let sem = render_surface.acquire();
        // let render_frame_idx = self.render_frame_idx;
        // let signal_semaphore = &render_surface.frame_signal_sems[render_frame_idx as usize];
        // let signal_fence = &render_surface.frame_fences[render_frame_idx as usize];
        graphics_queue
            .submit(&[&cmd_buffer], &[], &[&sem], None)
            .unwrap();

        render_context.release_cmd_buffer(cmd_buffer);

        render_surface.present(&mut render_context);
    }
}

fn render_post_update(renderer: ResMut<'_, Renderer>) {
    let mut render_context = RenderContext::new(&renderer);
    let cmd_buffer = render_context.acquire_cmd_buffer(QueueType::Graphics);
    cmd_buffer.begin().unwrap();
    cmd_buffer.end().unwrap();
    let frame_fence = renderer.frame_fence();
    let graphics_queue = renderer.queue(QueueType::Graphics);
    graphics_queue
        .submit(&[&cmd_buffer], &[], &[], Some(frame_fence))
        .unwrap();

    // let render_frame_idx = self.render_frame_idx;
    // let signal_fence = &self.frame_fences[render_frame_idx as usize];
    render_context.release_cmd_buffer(cmd_buffer);
}
