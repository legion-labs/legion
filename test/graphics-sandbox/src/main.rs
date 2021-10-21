use log::{LevelFilter};
use simple_logger::SimpleLogger;

use legion_app::App;
use legion_async::AsyncPlugin;
use legion_core::CorePlugin;
use legion_ecs::{prelude::*, system::IntoSystem};
use legion_input::InputPlugin;
use legion_presenter_window::PresenterWindowPlugin;
use legion_renderer::{GPUResourceFactory, RendererPlugin};
use legion_renderer::components::RenderSurface;
use legion_tao::TaoPlugin;
use legion_window::{
    WindowCloseRequested, WindowCreated,
    WindowPlugin, WindowResized, Windows,
};

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
        .add_system(on_window_created.system())
        .add_system(on_window_resized.system())
        .add_system(on_window_close_requested.system())
        .run();
}

fn on_window_created(
    mut commands: Commands,
    mut ev_wnd_created: EventReader<WindowCreated>,
    wnd_list: Res<Windows>,
    factory: Res<GPUResourceFactory>
) {
    for ev in ev_wnd_created.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();        
        commands.spawn().insert(
            RenderSurface::from_window(&factory, wnd)
        );
    }
}

fn on_window_resized(    
    mut ev_wnd_resized: EventReader<WindowResized>,
    wnd_list: Res<Windows>,
    factory: Res<GPUResourceFactory>,
    mut query: Query<&mut RenderSurface>,
) {
    for ev in ev_wnd_resized.iter() {
        let query_result = query.iter_mut().find(|x| x.id == ev.id);
        if let Some(mut render_surface) = query_result {
            let wnd = wnd_list.get(ev.id).unwrap();
            if (render_surface.width, render_surface.height) != (wnd.physical_width(), wnd.physical_height())                
            {
                render_surface.resize( 
                    &factory,
                    wnd.physical_width(),
                    wnd.physical_height() 
                );
            }
        }
    }
}

fn on_window_close_requested(
    mut commands: Commands,
    mut ev_wnd_destroyed: EventReader<WindowCloseRequested>,
    query: Query<(Entity, &RenderSurface)>,    
    
) {
    for ev in ev_wnd_destroyed.iter() {
        let query_result = query.iter().find(|x| x.1.id == ev.id);
        if let Some(query_result) = query_result {            
            commands.entity(query_result.0).despawn();
        }
    }
}
