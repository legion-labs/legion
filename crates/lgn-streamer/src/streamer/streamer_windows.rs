use std::sync::Arc;

use lgn_utils::HashMap;
use lgn_window::WindowId;
use webrtc::data_channel::RTCDataChannel;

#[derive(Default)]
pub(crate) struct StreamerWindows {
    pub window_id_to_data_channel: HashMap<WindowId, Arc<RTCDataChannel>>,
}

impl StreamerWindows {
    pub fn add_mapping(&mut self, window_id: WindowId, video_data_channel: Arc<RTCDataChannel>) {
        self.window_id_to_data_channel
            .insert(window_id, video_data_channel);
    }

    pub fn remove_mapping(&mut self, window_id: &WindowId) {
        self.window_id_to_data_channel.remove(window_id);
    }

    pub fn get_video_data_channel(&self, id: WindowId) -> Option<&Arc<RTCDataChannel>> {
        self.window_id_to_data_channel.get(&id)
    }
}
