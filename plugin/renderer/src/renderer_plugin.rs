use legion_app::Plugin;
use legion_ecs::{schedule::ParallelSystemDescriptorCoercion, system::{IntoSystem}};
use log::debug;

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut legion_app::App) {
        app.add_system(do_something.system().label("toto"));
    }
}

fn do_something() {            
    debug!( "do_something once per frame" );
}