use std::sync::Arc;

use anyhow::bail;
use lgn_input::{
    keyboard::KeyboardInput,
    mouse::{MouseButton, MouseButtonInput, MouseMotion, MouseWheel},
    touch::TouchInput,
    ElementState,
};
use lgn_math::Vec2;
use serde::Deserialize;
use webrtc::data_channel::RTCDataChannel;

use super::StreamID;

pub(crate) struct VideoStreamEvent {
    pub(crate) stream_id: StreamID,
    pub(crate) info: VideoStreamEventInfo,
    pub(crate) video_data_channel: Arc<RTCDataChannel>,
}

impl VideoStreamEvent {
    pub(crate) fn parse(
        stream_id: StreamID,
        video_data_channel: Arc<RTCDataChannel>,
        data: &[u8],
    ) -> anyhow::Result<Self> {
        Ok(Self {
            stream_id,
            info: serde_json::from_slice(data)?,
            video_data_channel,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct Color(pub [f32; 4]);

impl Default for Color {
    fn default() -> Self {
        // This is red.
        Self([1.0_f32, 0.0_f32, 0.0_f32, 1.0_f32])
    }
}

impl TryFrom<String> for Color {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.starts_with('#') {
            bail!("color values must start with #");
        }

        let mut bytes = hex::decode(value[1..].as_bytes())?;
        if bytes.len() == 3 {
            // append alpha if not specified
            bytes.push(0xff_u8);
        }

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
pub(crate) struct MouseButtonInputPayload {
    pub button: MouseButton,
    pub state: ElementState,
    pub pos: Vec2,
}

impl From<&MouseButtonInputPayload> for MouseButtonInput {
    fn from(MouseButtonInputPayload { button, state, pos }: &MouseButtonInputPayload) -> Self {
        Self {
            button: *button,
            state: *state,
            pos: *pos,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Input {
    MouseButtonInput(MouseButtonInput),
    MouseMotion(MouseMotion),
    MouseWheel(MouseWheel),
    TouchInput(TouchInput),
    KeyboardInput(KeyboardInput),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub(crate) enum VideoStreamEventInfo {
    #[serde(rename = "resize")]
    Resize { width: u32, height: u32 },
    #[serde(rename = "initialize")]
    Initialize {
        color: Color,
        width: u32,
        height: u32,
    },
    #[serde(rename = "speed")]
    Speed { speed: f32 },
    #[serde(rename = "input")]
    Input { input: Input },
}
