use legion_app::Plugin;
use legion_ecs::{schedule::ParallelSystemDescriptorCoercion, system::{IntoSystem}};
use log::trace;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, app: &mut legion_app::App) {
        app.add_system(consume_something.system().after("toto"));
    }
}

fn consume_something() {
    trace!( "consume_something once per frame" );
}