use crate::{components::AnimationClip, runtime_graph::node::Node};

#[derive(Clone)]
pub struct AnimationClipNode {
    pub id: usize,
    pub clip: AnimationClip,
}

impl Node for AnimationClipNode {
    fn update(&mut self, delta_time: f32) {
        self.clip.time_since_last_tick += delta_time;

        let current_key_frame_idx = self.clip.current_key_frame_index;

        // Changes frame when at exact key frame
        if AnimationClipNode::is_exact_key_frame(
            self.clip.time_since_last_tick,
            self.clip.duration_key_frames[current_key_frame_idx],
        ) {
            self.clip.time_since_last_tick -= self.clip.duration_key_frames[current_key_frame_idx];

            if self.clip.looping && current_key_frame_idx == self.clip.poses.len() - 1 {
                self.clip.current_key_frame_index = 0;
            } else {
                self.clip.current_key_frame_index += 1;
            }
        }
    }

    fn get_clip(&self) -> Option<&AnimationClip> {
        Some(&self.clip)
    }

    fn get_state_name(&self) -> Option<&String> {
        Some(&self.clip.name)
    }
}

impl AnimationClipNode {
    fn is_exact_key_frame(time_since_last_tick: f32, duration_current_key_frame: f32) -> bool {
        time_since_last_tick >= duration_current_key_frame
    }
}
