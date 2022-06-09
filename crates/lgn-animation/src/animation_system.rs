use crate::components::RuntimeAnimationClip;
use lgn_core::Time;
use lgn_ecs::prelude::{Query, Res};

pub(crate) fn update(
    mut animations: Query<'_, '_, &mut RuntimeAnimationClip>,
    time: Res<'_, Time>,
) {
    for mut animation in animations.iter_mut() {
        let delta_time = time.delta_seconds();

        animation.time_since_last_tick += delta_time;
        let current_key_frame_idx = animation.current_key_frame_index;

        // Changes pose when at exact key frame
        if is_exact_key_frame(
            animation.time_since_last_tick,
            animation.duration_key_frames[current_key_frame_idx as usize],
        ) {
            animation.time_since_last_tick = 0.0;

            if animation.looping && current_key_frame_idx == (animation.poses.len() - 1) as i32 {
                animation.current_key_frame_index = 0;
            } else {
                animation.current_key_frame_index += 1;
            }
        }
    }
    drop(animations);
}

fn is_exact_key_frame(time_since_last_tick: f32, duration_current_key_frame: f32) -> bool {
    time_since_last_tick / duration_current_key_frame >= 1.0
}
