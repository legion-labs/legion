use std::sync::Arc;

use lgn_app::Events;
use lgn_utils::HashMap;
use lgn_window::{Window, WindowCreated, WindowDescriptor, WindowId};
use webrtc::data_channel::RTCDataChannel;

use super::{Resolution, StreamID};

#[derive(Default)]
pub(crate) struct StreamerWindows {
    pub window_id_to_data_channel: HashMap<WindowId, Arc<RTCDataChannel>>,
    pub stream_id_to_window_id: HashMap<StreamID, WindowId>,
}

impl StreamerWindows {
    pub fn create_window(
        &mut self,
        stream_id: StreamID,
        resolution: Resolution,
        video_data_channel: Arc<RTCDataChannel>,
        window_created: &mut Events<WindowCreated>,
    ) -> Window {
        #[allow(clippy::cast_precision_loss)]
        let window_descriptor = WindowDescriptor {
            width: resolution.width() as f32,
            height: resolution.height() as f32,
            ..WindowDescriptor::default()
        };

        let window_id = WindowId::new();
        window_created.send(WindowCreated { id: window_id });

        self.window_id_to_data_channel
            .insert(window_id, video_data_channel);

        self.stream_id_to_window_id.insert(stream_id, window_id);

        Window::new(
            window_id,
            &window_descriptor,
            resolution.width(),
            resolution.height(),
            1.0,
            None,
        )
    }

    pub fn get_video_data_channel(&self, id: WindowId) -> Option<&Arc<RTCDataChannel>> {
        self.window_id_to_data_channel.get(&id)
    }

    pub fn get_window_id(&self, id: StreamID) -> Option<WindowId> {
        self.stream_id_to_window_id.get(&id).copied()
    }
}
