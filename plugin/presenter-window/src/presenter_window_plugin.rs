use legion_app::Plugin;
use legion_ecs::{schedule::ParallelSystemDescriptorCoercion, system::IntoSystem};
use legion_renderer::RendererSystemLabel;
use log::trace;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, app: &mut legion_app::App) {
        app.add_system(consume_something.system().after(RendererSystemLabel::Main));
    }
}

fn consume_something() {
    trace!("consume_something once per frame");
}
