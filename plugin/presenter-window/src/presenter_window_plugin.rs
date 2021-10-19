use legion_app::{App, Plugin};

// use std::slice::SliceIndex;

// use legion_app::{EventReader,  Plugin};
// use legion_ecs::{prelude::*};
// use legion_renderer::RendererSystemLabel;
// use legion_window::{WindowCreated, Windows};
// use log::trace;

#[derive(Default)]
pub struct PresenterWindowPlugin;

impl Plugin for PresenterWindowPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_system(on_window_msg.system());        
    }
}

// fn consume_something() {
//     trace!("consume_something once per frame");
// }

// // fn on_window_msg(windows: Res<Windows>, mut ev_levelup: EventReader<WindowCreated>) {
    
// //     for i in ev_levelup.iter() {
// //         let wnd = windows.get(i.id);
// //         if let Some(wnd) = wnd {
            
// //         }                
// //     }
// // }
