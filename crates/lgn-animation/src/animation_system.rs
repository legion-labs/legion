use crate::components::GraphDefinition;
use lgn_app::EventReader;
use lgn_core::Time;
use lgn_ecs::prelude::{Query, Res};
use lgn_input::keyboard::{KeyCode, KeyboardInput};

pub(crate) fn graph_update(
    mut graphs: Query<'_, '_, &mut GraphDefinition>,
    time: Res<'_, Time>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
) {
    for mut graph in graphs.iter_mut() {
        let delta_time = time.delta_seconds();
        let current_node_index = graph.current_node_index;

        // Check if keyboard input event
        for keyboard_input_event in keyboard_input_events.iter() {
            if let Some(key_code) = keyboard_input_event.key_code {
                if is_arrow(key_code) {
                    (*graph.nodes[current_node_index]).update_key_event(keyboard_input_event);
                }
            }
        }

        if keyboard_input_events.is_empty() {
            graph.current_node_index = 0;
        }

        // update the current node
        (*graph.nodes[current_node_index]).update_time(delta_time);
    }
    drop(graphs);
    drop(time);
}

pub(crate) fn clip_update(graphs: Query<'_, '_, &mut GraphDefinition>, time: Res<'_, Time>) {
    drop(graphs);
    drop(time);
}

fn is_arrow(key_code: KeyCode) -> bool {
    matches!(
        key_code,
        KeyCode::Left | KeyCode::Up | KeyCode::Right | KeyCode::Down
    )
}
/*
#[allow(clippy::needless_pass_by_value)]
fn update_input(
    egui: Res<'_, Egui>,
    mut keyboard_input_events: EventReader<'_, '_, KeyboardInput>,
) {
    for keyboard_input_event in keyboard_input_events.iter() {
        if let Some(key_code) = keyboard_input_event.key_code {
            if key_code == KeyCode::M && keyboard_input_event.state.is_pressed() {
                let mut inner = egui.inner.lock().unwrap();
                inner.enabled = !inner.enabled;
            }
        }
    }
}*/
