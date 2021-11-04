use legion_app::{CoreStage, Plugin};
use legion_ecs::{prelude::*, system::IntoSystem};

use crate::{components::RenderSurface, labels::RendererSystemLabel, Renderer};

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new().unwrap();

        app.insert_resource(renderer);

        // Pre-Update
        app.add_system_to_stage(CoreStage::PreUpdate, render_pre_update.system());
        
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

fn render_update(
    mut renderer: ResMut<'_, Renderer>,
    mut q_render_surfaces: Query<'_, '_, &mut RenderSurface>,
) {
    renderer.update(&mut q_render_surfaces);
}

fn render_post_update(mut renderer: ResMut<'_, Renderer>) {
    renderer.end_frame();
}
