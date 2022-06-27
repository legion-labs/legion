use lgn_app::AppExit;
use lgn_ecs::prelude::*;
use lgn_input::{keyboard::KeyCode, Input};

use crate::{Window, WindowCloseRequested, WindowFocused, WindowId, Windows};

/// Exit the application when there are no open windows.
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_all_closed(
    mut app_exit_events: EventWriter<'_, '_, AppExit>,
    windows: Res<'_, Windows>,
) {
    if windows.iter().count() == 0 {
        app_exit_events.send(AppExit);
    }

    drop(windows);
}

/// Close windows in response to [`WindowCloseRequested`] (e.g.  when the close button is pressed).
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behaviour, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn close_when_requested(
    mut windows: ResMut<'_, Windows>,
    mut closed: EventReader<'_, '_, WindowCloseRequested>,
) {
    for event in closed.iter() {
        windows.get_mut(event.id).map(Window::close);
    }
}

/// Close the focused window whenever the escape key (<kbd>Esc</kbd>) is pressed
///
/// This is useful for examples or prototyping.
pub fn close_on_esc(
    mut focused: Local<'_, Option<WindowId>>,
    mut focused_events: EventReader<'_, '_, WindowFocused>,
    mut windows: ResMut<'_, Windows>,
    input: Res<'_, Input<KeyCode>>,
) {
    // TODO: Track this in e.g. a resource to ensure consistent behaviour across similar systems
    for event in focused_events.iter() {
        *focused = event.focused.then(|| event.id);
    }

    if let Some(focused) = &*focused {
        if input.just_pressed(KeyCode::Escape) {
            if let Some(window) = windows.get_mut(*focused) {
                window.close();
            }
        }
    }

    drop(input);
}
