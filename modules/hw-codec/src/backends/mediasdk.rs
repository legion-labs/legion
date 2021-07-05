use crate::{CpuBuffer, GpuImage, VideoProcessor};

use super::EncoderConfig;

/// `MediaSdk` Encoder Config
#[derive(Debug)]
pub struct MediaSdkEncoderConfig {}

/// `MediaSdk` Encoder
#[derive(Debug)]
pub struct MediaSdkEncoder {}

impl VideoProcessor for MediaSdkEncoder {
    type Input = GpuImage;
    type Output = CpuBuffer;
    type Config = MediaSdkEncoderConfig;

    fn submit_input(&self, _input: &Self::Input) -> Result<(), crate::Error> {
        Ok(())
    }

    fn query_output(&self) -> Result<Self::Output, crate::Error> {
        Ok(CpuBuffer(Vec::new()))
    }

    fn new(_config: Self::Config) -> Option<Self> {
        Some(Self {})
    }
}

impl Default for MediaSdkEncoderConfig {
    fn default() -> Self {
        Self {}
    }
}

impl From<EncoderConfig> for MediaSdkEncoderConfig {
    fn from(_: EncoderConfig) -> Self {
        Self {}
    }
}
