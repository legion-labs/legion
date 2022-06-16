use lgn_core::Time;

use crate::components::AnimationClip;

use super::node_state_machine::StateInfo;

pub trait Node: Sync + Send {
    fn update(&mut self, time: f32) {}

    fn get_active_state(&mut self) -> Option<&mut StateInfo> {
        None
    }

    fn get_clip(&mut self) -> Option<&AnimationClip> {
        None
    }
}

impl dyn Node {}
