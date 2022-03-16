use super::EncoderConfig;
use crate::{CpuBuffer, GpuImage, VideoProcessor};

mod cuda;
mod loader;

pub use cuda::{CuContext, CuDevice};
pub use loader::{CudaApi, NvEncApi};

/// Nvenc Encoder Config
#[derive(Debug)]
pub struct NvEncEncoderConfig {}

/// Nvenc Encoder
#[derive(Default, Debug)]
pub struct NvEncEncoder {}

impl VideoProcessor for NvEncEncoder {
    type Input = GpuImage;
    type Output = CpuBuffer;
    type Config = NvEncEncoderConfig;

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

impl From<EncoderConfig> for NvEncEncoderConfig {
    fn from(_: EncoderConfig) -> Self {
        Self {}
    }
}
