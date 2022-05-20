use lgn_ecs::event::EventWriter;
use lgn_ecs::system::{NonSend, NonSendMut};
use lgn_input::{gamepad::GamepadEventRaw, prelude::*};

use gilrs::{EventType, Gilrs};

use crate::converter::{convert_axis, convert_button, convert_gamepad_id};

pub fn gilrs_event_startup_system(
    gilrs: NonSend<'_, Gilrs>,
    mut events: EventWriter<'_, '_, GamepadEventRaw>,
) {
    for (id, _) in gilrs.gamepads() {
        events.send(GamepadEventRaw(
            convert_gamepad_id(id),
            GamepadEventType::Connected,
        ));
    }

    drop(gilrs);
}

pub fn gilrs_event_system(
    mut gilrs: NonSendMut<'_, Gilrs>,
    mut events: EventWriter<'_, '_, GamepadEventRaw>,
) {
    while let Some(gilrs_event) = gilrs.next_event() {
        match gilrs_event.event {
            EventType::Connected => {
                events.send(GamepadEventRaw(
                    convert_gamepad_id(gilrs_event.id),
                    GamepadEventType::Connected,
                ));
            }
            EventType::Disconnected => {
                events.send(GamepadEventRaw(
                    convert_gamepad_id(gilrs_event.id),
                    GamepadEventType::Disconnected,
                ));
            }
            EventType::ButtonChanged(gilrs_button, value, _) => {
                if let Some(button_type) = convert_button(gilrs_button) {
                    events.send(GamepadEventRaw(
                        convert_gamepad_id(gilrs_event.id),
                        GamepadEventType::ButtonChanged(button_type, value),
                    ));
                }
            }
            EventType::AxisChanged(gilrs_axis, value, _) => {
                if let Some(axis_type) = convert_axis(gilrs_axis) {
                    events.send(GamepadEventRaw(
                        convert_gamepad_id(gilrs_event.id),
                        GamepadEventType::AxisChanged(axis_type, value),
                    ));
                }
            }
            _ => (),
        };
    }
    gilrs.inc();
}
