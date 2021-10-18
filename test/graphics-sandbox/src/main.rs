use log::LevelFilter;
use simple_logger::SimpleLogger;

use legion_app::App;
use legion_async::AsyncPlugin;
use legion_core::CorePlugin;
use legion_input::InputPlugin;
use legion_presenter_window::PresenterWindowPlugin;
use legion_renderer::RendererPlugin;
use legion_tao::TaoPlugin;
use legion_window::WindowPlugin;

fn main() {
    let logger = Box::new(SimpleLogger::new().with_level(LevelFilter::Debug));
    logger.init().unwrap();

    App::new()
        .add_plugin(CorePlugin::default())        
        .add_plugin(AsyncPlugin {})        
        .add_plugin(WindowPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(TaoPlugin::default())
        .add_plugin(RendererPlugin::default())
        .add_plugin(PresenterWindowPlugin::default())
        .run();
}
