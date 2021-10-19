use log::{LevelFilter};
use simple_logger::SimpleLogger;

use legion_app::App;
use legion_async::AsyncPlugin;
use legion_core::CorePlugin;
use legion_ecs::{prelude::*, system::IntoSystem};
use legion_input::InputPlugin;
use legion_presenter_window::PresenterWindowPlugin;
use legion_renderer::{RenderOutput, RendererPlugin};
use legion_tao::TaoPlugin;
use legion_window::{
    CreateWindow, WindowCloseRequested, WindowCreated,
    WindowPlugin, WindowResized, Windows,
};

struct FrameCount (u32);

fn main() {
    let logger = Box::new(SimpleLogger::new().with_level(LevelFilter::Debug));
    logger.init().unwrap();

    App::new()
        .insert_resource(FrameCount(0))
        .add_plugin(CorePlugin::default())
        .add_plugin(AsyncPlugin {})
        .add_plugin(WindowPlugin::default())
        .add_plugin(InputPlugin::default())
        .add_plugin(TaoPlugin::default())
        .add_plugin(RendererPlugin::default())
        .add_plugin(PresenterWindowPlugin::default())
        .add_startup_system(on_startup.system())
        .add_system(on_window_created.system())
        .add_system(on_window_resized.system())
        .add_system(on_window_close_requested.system())
        .run();
}

fn on_startup(mut _ev_writer: EventWriter<CreateWindow>) {    
}

fn on_window_created(
    mut commands: Commands,
    mut ev_wnd_created: EventReader<WindowCreated>,
    wnd_list: Res<Windows>,
) {
    for ev in ev_wnd_created.iter() {
        let wnd = wnd_list.get(ev.id).unwrap();        
        commands.spawn().insert(RenderOutput {
            id: ev.id,
            width: wnd.physical_width(),
            height: wnd.physical_height(),
        });
    }
}

fn on_window_resized(    
    mut ev_wnd_resized: EventReader<WindowResized>,
    wnd_list: Res<Windows>,
    mut query: Query<&mut RenderOutput>,
) {
    for ev in ev_wnd_resized.iter() {
        let query_result = query.iter_mut().find(|x| x.id == ev.id);
        if let Some(mut render_output) = query_result {
            let wnd = wnd_list.get(ev.id).unwrap();
            if (render_output.width,render_output.height) != (wnd.physical_width(), wnd.physical_height())                
            {
                *render_output = RenderOutput {
                    id: ev.id,
                    width: wnd.physical_width(),
                    height: wnd.physical_height(),
                }
            }
        }
    }
}

fn on_window_close_requested(
    mut commands: Commands,
    mut ev_wnd_destroyed: EventReader<WindowCloseRequested>,
    query: Query<(Entity, &RenderOutput)>,    
    
) {
    for ev in ev_wnd_destroyed.iter() {
        let query_result = query.iter().find(|x| x.1.id == ev.id);
        if let Some(query_result) = query_result {            
            commands.entity(query_result.0).despawn();
        }
    }
}
