use lgn_graphics_api::DeviceContext;

use crate::stream_encoder::StreamEncoder;

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
    pub work_queue: StreamEncoder,
    pub width: u32,
    pub height: u32,
}
