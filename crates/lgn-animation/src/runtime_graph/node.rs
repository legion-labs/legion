use lgn_input::keyboard::KeyboardInput;

use crate::components::AnimationClip;

use super::node_state_machine::StateInfo;

pub trait Node: Sync + Send {
    fn update_time(&mut self, _time: f32);

    fn update_key_event(&mut self, _key_event: &KeyboardInput) {}

    fn get_active_state(&self) -> Option<&StateInfo> {
        None
    }

    fn get_clip(&self) -> Option<&AnimationClip> {
        None
    }

    fn get_state_name(&self) -> Option<&String> {
        None
    }
}

impl dyn Node {}
