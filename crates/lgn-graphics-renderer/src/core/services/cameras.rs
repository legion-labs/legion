use lgn_transform::prelude::GlobalTransform;

use crate::components::CameraComponent;

#[derive(Default)]
pub struct RenderCamera {}

impl From<(&GlobalTransform, &CameraComponent)> for RenderCamera {
    fn from(_: (&GlobalTransform, &CameraComponent)) -> Self {
        RenderCamera {}
    }
}
