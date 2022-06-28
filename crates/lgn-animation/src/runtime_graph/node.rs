use crate::components::AnimationClip;
use crate::runtime_graph::node_state_machine::StateInfo;

pub trait Node: Sync + Send {
    fn update(&mut self, _time: f32);

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
