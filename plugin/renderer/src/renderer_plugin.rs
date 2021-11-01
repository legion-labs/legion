use graphics_api::{CommandBuffer, DefaultApi, ResourceState, TextureBarrier};
use legion_app::Plugin;
use legion_ecs::{prelude::*, system::IntoSystem};

use crate::{components::RenderSurface, Renderer};

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

fn render(mut renderer: ResMut<Renderer>, mut outputs: Query<(Entity, &mut RenderSurface)>) {
    renderer.begin_frame();

    let cmd_buffer = renderer.get_cmd_buffer();

    for (_, mut render_surface) in outputs.iter_mut() {
        render_surface.transition_to(cmd_buffer, ResourceState::RENDER_TARGET);

        {
            let render_pass = &render_surface.test_renderpass;
            render_pass.render(&renderer, &render_surface, cmd_buffer);
        }

        // cmd_buffer
        //     .cmd_resource_barrier(
        //         &[],
        //         &[TextureBarrier::<DefaultApi>::state_transition(
        //             render_target,
        //             ResourceState::RENDER_TARGET,
        //             ResourceState::SHADER_RESOURCE | ResourceState::COPY_SRC,
        //         )],
        //     )
        //     .unwrap();
    }

    renderer.end_frame();
}
