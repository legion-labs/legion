use legion_app::{EventReader, Plugin};
use legion_ecs::{prelude::*};
use legion_window::{WindowCloseRequested, WindowCreated, WindowResized, Windows};
use log::debug;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, app: &mut legion_app::App) {
        app.add_system_set(
            SystemSet::new()
            .with_system(on_window_created.system())
            .with_system(on_window_resized.system())
            .with_system(on_window_close_requested.system())
            .after("toto")
        );
    }
}

fn on_window_created(mut ev_window: EventReader<WindowCreated>, windows: Res<Windows> ) {

    for ev in ev_window.iter() {
        
        let window = windows.get(ev.id).unwrap();
        debug!( "window {} created", window.id() );
    }
}

fn on_window_resized(mut ev_window: EventReader<WindowResized>, windows: Res<Windows> ) {

    for ev in ev_window.iter() {
        
        let window = windows.get(ev.id).unwrap();
        debug!( "window {} resized {} {}", window.id(), ev.width, ev.height );
    }
}

fn on_window_close_requested(mut ev_window: EventReader<WindowCloseRequested>, windows: Res<Windows> ) {

    for ev in ev_window.iter() {
        
        let window = windows.get(ev.id).unwrap();
        debug!( "window {} close requested", window.id() );
    }
}
