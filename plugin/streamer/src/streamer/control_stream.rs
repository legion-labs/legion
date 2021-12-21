use std::sync::Arc;

use anyhow::{bail, Context, Result};
use lgn_ecs::prelude::*;
use lgn_telemetry::prelude::*;
use serde::Serialize;
use webrtc::data_channel::RTCDataChannel;

use lgn_telemetry::error;

#[derive(Debug, Serialize)]
#[serde(tag = "control_msg")]
enum ControlStreamMessage {
    #[serde(rename = "hello")]
    Hello { process_id: String },
}

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

    pub(crate) fn say_hello(&mut self) -> Result<impl std::future::Future<Output = ()> + 'static> {
        if let Some(process_id) = get_process_id() {
            let message = serde_json::to_string(&ControlStreamMessage::Hello { process_id })
                .with_context(|| "Error formatting hello message")?;
            let buffer = bytes::Bytes::copy_from_slice(message.as_bytes());
            let control_data_channel = Arc::clone(&self.control_data_channel);

            Ok(async move {
                if let Err(err) = control_data_channel.send(&buffer).await {
                    error!("Error sending hello message on control stream: {}", err);
                }
            })
        } else {
            bail!("Error getting telemetry process id");
        }
    }
}
