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
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut lgn_app::App) {
        let renderer = Renderer::new().unwrap();

        app.insert_resource(renderer);

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update);
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
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
    q_drawables: Query<'_, '_, (&Transform, &StaticMesh)>,
    task_pool: Res<'_, crate::RenderTaskPool>,
) {
    let mut render_context = RenderContext::new(&renderer);
    let q_drawables = q_drawables
        .iter()
        .collect::<Vec<(&Transform, &StaticMesh)>>();
    let graphics_queue = renderer.queue(QueueType::Graphics);

    renderer.flush_update_jobs(&mut render_context, &graphics_queue);

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
