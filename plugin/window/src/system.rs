use crate::WindowCloseRequested;
use legion_app::{AppExit, EventReader, EventWriter};

pub fn exit_on_window_close_system(
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
    mut window_close_requested_events: EventReader<'_, '_, WindowCloseRequested>,
) {
    if window_close_requested_events.iter().next().is_some() {
        app_exit_events.send(AppExit);
    }
}

