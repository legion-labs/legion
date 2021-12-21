pub use bevy_input::mouse::{MouseButton, MouseMotion, MouseWheel};
use serde::{Deserialize, Serialize};

use crate::ElementState;
use lgn_math::Vec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: ElementState,
    pub pos: Vec2,
}
