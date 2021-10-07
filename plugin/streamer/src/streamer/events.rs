use super::StreamID;
use serde::Deserialize;

#[derive(Debug)]
pub(crate) struct VideoStreamEvent {
    pub(crate) stream_id: StreamID,
    pub(crate) info: VideoStreamEventInfo,
}

impl VideoStreamEvent {
    pub(crate) fn parse(stream_id: StreamID, data: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {
            stream_id,
            info: serde_json::from_slice(data)?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub(crate) enum VideoStreamEventInfo {
    #[serde(rename = "resize")]
    Resize { width: u32, height: u32 },
    #[serde(rename = "hue")]
    Hue { hue: f32 },
    #[serde(rename = "speed")]
    Speed { speed: f32 },
}
