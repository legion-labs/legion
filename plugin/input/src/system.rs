use crate::{
    keyboard::{KeyCode, KeyboardInput},
    ElementState,
};
use legion_app::AppExit;
use legion_ecs::prelude::{EventReader, EventWriter};

/// Sends the `AppExit` event whenever the "esc" key is pressed.
pub fn exit_on_esc_system(
    mut keyboard_input_events: EventReader<'_, KeyboardInput>,
    mut app_exit_events: EventWriter<'_, AppExit>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::Escape {
                app_exit_events.send(AppExit);
            }
        }
    }
}
