use super::labels::*;
use graphics_renderer::Renderer;
use legion_app::Plugin;
use legion_ecs::{prelude::*, schedule::ParallelSystemDescriptorCoercion, system::IntoSystem};

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        let renderer = Renderer::new(1024, 1024);
        app.insert_resource(renderer);
        app.add_system(render.system().label(RendererSystemLabel::Main));
    }
}

fn render(mut renderer: ResMut<Renderer>) {
    renderer.render();
}
