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
pub struct Color(pub [f32; 4]);

impl Default for Color {
    fn default() -> Self {
        // This is red.
        Self(
            [1.0f32, 0.0f32, 0.0f32, 1.0f32]
        )
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

        Ok(Self(array))
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
