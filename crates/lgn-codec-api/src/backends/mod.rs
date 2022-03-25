use lgn_graphics_api::DeviceContext;

use self::nvenc::NvEncEncoder;
use crate::{CpuBuffer, GpuBuffer, VideoProcessor};

/// Null Encoder/Decoder
pub mod null;
/// `NvEnc` Encoder/Decoder
pub mod nvenc;
/// `NvEnc` Encoder/Decoder
pub mod openh264;

/// The hardware we want to run on, this maps to Amf/NvEnc/MediaSdk
#[derive(Debug, PartialEq)]
pub enum CodecHardware {
    /// Amd hardware, uses Amf library
    Amd,
    /// Nvidia hardware, uses NvEnc library
    Nvidia,
    /// Intel hardware, uses MediaSdk library
    Intel,
}

/// Graphics Context for initialization
pub enum GraphicsConfig {
    /// Vulkan config, not all values are used by all HW encoders
    Vulkan(DeviceContext),
}

/// Generic configuration that applies to all encoders
/// All supported Encoder implement a conversion from the generic
/// config to the hardware specific one
pub struct EncoderConfig {
    pub hardware: CodecHardware,
    pub gfx_config: DeviceContext,
    pub width: u32,
    pub height: u32,
}

/// Generic encoder,
pub enum Encoder {
    /// `NvEnc` Encoder
    NvEnc(NvEncEncoder),
}

impl VideoProcessor for Encoder {
    type Input = GpuBuffer;
    type Output = CpuBuffer;
    type Config = EncoderConfig;

    fn submit_input(&self, _input: &Self::Input) -> Result<(), crate::Error> {
        Ok(())
    }

    fn query_output(&self) -> Result<Self::Output, crate::Error> {
        Ok(CpuBuffer(Vec::new()))
    }

    fn new(config: Self::Config) -> Option<Self> {
        if config.hardware == CodecHardware::Nvidia {
            NvEncEncoder::new(config.into()).map(Encoder::NvEnc)
        } else {
            None
        }
    }
}
