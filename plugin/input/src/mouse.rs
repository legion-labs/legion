use lgn_ecs::{event::EventReader, system::ResMut};
use lgn_math::Vec2;

use crate::{ElementState, Input};

/// A mouse button input event
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: ElementState,
    pub pos: Vec2,
}

/// A button on a mouse device
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MouseButton {
    Left, // TODO: we may need to change this notation to Primary/Secondary and match it with the OS settings in case left-handed user changed it
    Right,
    Middle,
    Other(u16),
}

/// A mouse motion event
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseMotion {
    pub delta: Vec2,
}

/// Unit of scroll
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MouseScrollUnit {
    Line,
    Pixel,
}

/// A mouse scroll wheel event, where x represents horizontal scroll and y represents vertical
/// scroll.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseWheel {
    pub unit: MouseScrollUnit,
    pub x: f32,
    pub y: f32,
}

/// Updates the Input<MouseButton> resource with the latest `MouseButtonInput` events
pub fn mouse_button_input_system(
    mut mouse_button_input: ResMut<'_, Input<MouseButton>>,
    mut mouse_button_input_events: EventReader<'_, '_, MouseButtonInput>,
) {
    mouse_button_input.clear();
    for event in mouse_button_input_events.iter() {
        match event.state {
            ElementState::Pressed => mouse_button_input.press(event.button),
            ElementState::Released => mouse_button_input.release(event.button),
        }
    }
}
