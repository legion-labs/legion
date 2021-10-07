use legion_ecs::prelude::*;

use std::sync::Arc;
use webrtc::data::data_channel::RTCDataChannel;

#[derive(Component)]
#[component(storage = "Table")]
pub(crate) struct ControlStream {
    #[allow(dead_code)]
    control_data_channel: Arc<RTCDataChannel>,
}

impl ControlStream {
    pub(crate) fn new(control_data_channel: Arc<RTCDataChannel>) -> Self {
        Self {
            control_data_channel,
        }
    }
}
