use lgn_ecs::{event::EventReader, system::ResMut};
use lgn_math::Vec2;

use crate::{ButtonState, Input};

/// A mouse button input event.
///
/// This event is the translated version of the `WindowEvent::MouseInput` from the `winit` crate.
///
/// ## Usage
///
/// The event is read inside of the [`mouse_button_input_system`](crate::mouse::mouse_button_input_system)
/// to update the [`Input<MouseButton>`](crate::Input<MouseButton>) resource.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseButtonInput {
    /// The mouse button assigned to the event.
    pub button: MouseButton,
    /// The pressed state of the button.
    pub state: ButtonState,
    pub pos: Vec2,
}

/// A button on a mouse device.
///
/// ## Usage
///
/// It is used as the generic `T` value of an [`Input`](crate::Input) to create a `legion`
/// resource.
///
/// ## Updating
///
/// The resource is updated inside of the [`mouse_button_input_system`](crate::mouse::mouse_button_input_system).
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MouseButton {
    /// The left mouse button.
    Left, /* TODO: we may need to change this notation to Primary/Secondary and match it with
           * the OS settings in case left-handed user changed it */
    /// The right mouse button.
    Right,
    /// The middle mouse button.
    Middle,
    /// Another mouse button with the associated number.
    Other(u16),
}

/// A mouse motion event.
///
/// This event is the translated version of the `DeviceEvent::MouseMotion` from the `winit` crate.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseMotion {
    /// The delta of the previous and current mouse positions.
    pub delta: Vec2,
}

/// The scroll unit.
///
/// Describes how a value of a [`MouseWheel`](crate::mouse::MouseWheel) event has to be interpreted.
///
/// The value of the event can either be interpreted as the amount of lines or the amount of pixels
/// to scroll.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum MouseScrollUnit {
    /// The line scroll unit.
    ///
    /// The delta of the associated [`MouseWheel`](crate::mouse::MouseWheel) event corresponds
    /// to the amount of lines or rows to scroll.
    Line,
    /// The pixel scroll unit.
    ///
    /// The delta of the associated [`MouseWheel`](crate::mouse::MouseWheel) event corresponds
    /// to the amount of pixels to scroll.
    Pixel,
}

/// A mouse wheel event.
///
/// This event is the translated version of the `WindowEvent::MouseWheel` from the `winit` crate.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct MouseWheel {
    /// The mouse scroll unit.
    pub unit: MouseScrollUnit,
    /// The horizontal scroll value.
    pub x: f32,
    /// The vertical scroll value.
    pub y: f32,
}

/// Updates the [`Input<MouseButton>`] resource with the latest [`MouseButtonInput`] events.
///
/// ## Differences
///
/// The main difference between the [`MouseButtonInput`] event and the [`Input<MouseButton>`] resource is that
/// the latter has convenient functions like [`Input::pressed`], [`Input::just_pressed`] and [`Input::just_released`].
pub fn mouse_button_input_system(
    mut mouse_button_input: ResMut<'_, Input<MouseButton>>,
    mut mouse_button_input_events: EventReader<'_, '_, MouseButtonInput>,
) {
    mouse_button_input.clear();
    for event in mouse_button_input_events.iter() {
        match event.state {
            ButtonState::Pressed => mouse_button_input.press(event.button),
            ButtonState::Released => mouse_button_input.release(event.button),
        }
    }
}
