use graphics_api::ResourceState;
use legion_app::Plugin;
use legion_ecs::{prelude::*, system::IntoSystem};

use crate::{components::RenderSurface, FrameContext, Renderer};

use super::labels::*;

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new();

        app.insert_resource(renderer);
        app.add_system_set(
            SystemSet::new()
                .with_system(render.system())
                .label(RendererSystemLabel::Main),
        );
    }
}

fn render(renderer: ResMut<Renderer>, mut outputs: Query<(Entity, &mut RenderSurface)>) {
    let frame_context = FrameContext::new(renderer.into_inner());

    let cmd_buffer = frame_context.renderer().get_cmd_buffer();

    for (_, mut render_surface) in outputs.iter_mut() {
        render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

        {
            let render_pass = &render_surface.test_renderpass;
            render_pass.render(frame_context.renderer(), &render_surface, cmd_buffer);
        }
    }
}
