use std::convert::{TryFrom, TryInto};

use super::StreamID;
use anyhow::bail;
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

#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct Color(pub legion_graphics_api::ColorClearValue);

impl Default for Color {
    fn default() -> Self {
        // This is red.
        let mut c = legion_graphics_api::ColorClearValue::default();
        c.0[0] = 1.0;
        c.0[3] = 1.0;

        Self(c)
    }
}

impl TryFrom<String> for Color {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.starts_with('#') {
            bail!("color values must start with #");
        }

        let bytes = hex::decode(value[1..].as_bytes())?;

        if bytes.len() != 4 {
            bail!("expected `#RGBA` but got `{}`", value);
        }

        let array: [f32; 4] = match bytes
            .into_iter()
            .map(|x| f32::from(x) / 255.0)
            .collect::<Vec<f32>>()
            .try_into()
        {
            Ok(v) => v,
            Err(v) => bail!("expected #RGBA but got vector with {} element(s)", v.len()),
        };

        Ok(Self(legion_graphics_api::ColorClearValue(array)))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub(crate) enum VideoStreamEventInfo {
    #[serde(rename = "resize")]
    Resize { width: u32, height: u32 },
    #[serde(rename = "color")]
    Color { id: String, color: Color },
    #[serde(rename = "speed")]
    Speed { id: String, speed: f32 },
}
