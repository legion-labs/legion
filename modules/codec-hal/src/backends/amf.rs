use crate::{CpuBuffer, GpuImage, VideoProcessor};

use super::{EncoderConfig, GraphicsConfig};

/// Amf Encoder Config
pub struct AmfEncoderConfig {
    gfx_config: GraphicsConfig,
}

/// Amf Encoder
pub struct AmfEncoder {
    runtime_version: u64,
    #[allow(dead_code)] // WIP
    context: amf::factory::Context,
    #[allow(dead_code)] // WIP
    component: amf::factory::Component,
}

impl AmfEncoder {
    /// Get the runtime version of the amf lib bundled with the drivers
    pub fn runtime_version(&self) -> u64 {
        self.runtime_version
    }
}

impl VideoProcessor for AmfEncoder {
    type Input = GpuImage;
    type Output = CpuBuffer;
    type Config = AmfEncoderConfig;

    fn submit_input(&self, _input: &Self::Input) -> Result<(), crate::Error> {
        //self.component.submit_input(data)
        Ok(())
    }

    fn query_output(&self) -> Result<Self::Output, crate::Error> {
        Ok(CpuBuffer(Vec::new()))
    }

    fn new(config: Self::Config) -> Option<Self> {
        if let Some(runtime_version) = amf::runtime_version() {
            let mut context = amf::factory::create_context().expect("context creation failed");
            // A component might not exist in the loaded version of the library
            match config.gfx_config {
                GraphicsConfig::Vulkan(instance, physical_device, device) => {
                    if context
                        .init_vulkan(instance, physical_device, device)
                        .is_err()
                    {
                        return None;
                    }
                }
            }
            if let Ok(component) =
                amf::factory::create_component(&context, amf::constants::avc::VIDEO_ENCODER_AVC)
            {
                Some(Self {
                    runtime_version,
                    context,
                    component,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for AmfEncoderConfig {
    fn default() -> Self {
        Self {
            gfx_config: GraphicsConfig::Vulkan(
                ash::vk::Instance::null(),
                ash::vk::PhysicalDevice::null(),
                ash::vk::Device::null(),
            ),
        }
    }
}

impl From<EncoderConfig> for AmfEncoderConfig {
    fn from(config: EncoderConfig) -> Self {
        Self {
            gfx_config: config.gfx_config,
        }
    }
}
