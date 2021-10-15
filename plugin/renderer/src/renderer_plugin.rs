use legion_app::Plugin;
use legion_ecs::{schedule::ParallelSystemDescriptorCoercion, system::{IntoSystem}};
use log::trace;

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        app.add_system(do_something.system().label("toto"));
    }
}

fn do_something() {            
    trace!( "do_something once per frame" );
}