#![allow(unsafe_code)]

use std::sync::{Arc, Mutex};

use super::EncoderConfig;
use crate::{CpuBuffer, GpuImage, VideoProcessor};

mod cuda;
mod loader;

pub use cuda::{CuContext, CuDevice};
pub use loader::{CudaApi, NvEncApi};

/// Nvenc Encoder Config
#[derive(Debug)]
pub struct NvEncEncoderConfig {
    width: u32,
    height: u32,
}

pub struct NvEncEncoderInner {
    cuda_context: CuContext,
}

pub struct NvEncEncoder {
    inner: Arc<Mutex<NvEncEncoderInner>>,
}

impl NvEncEncoder {}

#[allow(unsafe_code)]
unsafe impl Send for NvEncEncoder {}
#[allow(unsafe_code)]
unsafe impl Sync for NvEncEncoder {}

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

    fn new(config: Self::Config) -> Option<Self> {
        if let Some(context) = CudaApi::load()
            .and_then(CuDevice::new)
            .and_then(|device| CuContext::new(&device))
        {
            if let Some(nvenc) = NvEncApi::load() {
                let mut function_list = nvenc_sys::NV_ENCODE_API_FUNCTION_LIST::default();

                unsafe {
                    (nvenc.create_instance)(std::ptr::addr_of_mut!(function_list));
                }

                let mut open_session_ex_params = nvenc_sys::NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS {
                    version: nvenc_sys::NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER,
                    deviceType: nvenc_sys::NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
                    device: (*context.cuda_context()).cast::<std::ffi::c_void>(),
                    apiVersion: nvenc_sys::NVENCAPI_VERSION,
                    ..nvenc_sys::NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS::default()
                };
                let mut encoder: *mut ::std::os::raw::c_void = std::ptr::null_mut();

                unsafe {
                    (function_list.nvEncOpenEncodeSessionEx.unwrap())(
                        std::ptr::addr_of_mut!(open_session_ex_params),
                        std::ptr::addr_of_mut!(encoder),
                    );
                }

                let caps_to_query = nvenc_sys::NV_ENC_CAPS_PARAM {
                    version: nvenc_sys::NV_ENC_CAPS_PARAM_VER,
                    capsToQuery: nvenc_sys::_NV_ENC_CAPS::NV_ENC_CAPS_ASYNC_ENCODE_SUPPORT,
                    ..nvenc_sys::NV_ENC_CAPS_PARAM::default()
                };

                let mut present_config = nvenc_sys::NV_ENC_PRESET_CONFIG {
                    version: nvenc_sys::NV_ENC_PRESET_CONFIG_VER,
                    ..nvenc_sys::NV_ENC_PRESET_CONFIG::default()
                };

                unsafe {
                    (function_list.nvEncGetEncodePresetConfigEx.unwrap())(
                        encoder,
                        nvenc_sys::NV_ENC_CODEC_H264_GUID,
                        nvenc_sys::NV_ENC_PRESET_P3_GUID,
                        nvenc_sys::NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
                        std::ptr::addr_of_mut!(present_config),
                    );
                }
                present_config
                    .presetCfg
                    .encodeCodecConfig
                    .h264Config
                    .idrPeriod = nvenc_sys::NVENC_INFINITE_GOPLENGTH;

                unsafe {
                    let mut create_encode_params = nvenc_sys::NV_ENC_INITIALIZE_PARAMS {
                        version: nvenc_sys::NV_ENC_INITIALIZE_PARAMS_VER,
                        encodeGUID: nvenc_sys::NV_ENC_CODEC_H264_GUID,
                        presetGUID: nvenc_sys::NV_ENC_PRESET_P3_GUID,
                        encodeWidth: config.width,
                        encodeHeight: config.height,
                        darWidth: config.width,
                        darHeight: config.height,
                        frameRateNum: 60,
                        frameRateDen: 1,
                        enableEncodeAsync: 0,
                        enablePTD: 1,
                        encodeConfig: std::ptr::addr_of_mut!(present_config.presetCfg),
                        maxEncodeWidth: config.width,
                        maxEncodeHeight: config.height,
                        tuningInfo: nvenc_sys::NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
                        ..nvenc_sys::NV_ENC_INITIALIZE_PARAMS::default()
                    };
                    (function_list.nvEncInitializeEncoder.unwrap())(
                        encoder,
                        std::ptr::addr_of_mut!(create_encode_params),
                    );
                }

                let register_resource = nvenc_sys::NV_ENC_REGISTER_RESOURCE {
                    version: nvenc_sys::NV_ENC_REGISTER_RESOURCE_VER,
                    resourceType: nvenc_sys::_NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_CUDADEVICEPTR,
                    width: config.width,
                    height: config.height,
                    resourceToRegister: pBuffer,
                    pitch: pitch,
                    bufferFormat: bufferFormat,
                    bufferUsage: bufferUsage,
                    pInputFencePoint: pInputFencePoint,
                    pOutputFencePoint: pOutputFencePoint,
                    ..nvenc_sys::NV_ENC_REGISTER_RESOURCE::default()
                };

                (function_list.nvEncRegisterResource.unwrap())(
                    encoder,
                    std::ptr::addr_of_mut!(register_resource),
                );

                return Some(Self {
                    inner: Arc::new(Mutex::new(NvEncEncoderInner {
                        cuda_context: context,
                    })),
                });
            }
        }
        None
    }
}

impl From<EncoderConfig> for NvEncEncoderConfig {
    fn from(config: EncoderConfig) -> Self {
        Self {
            width: config.width,
            height: config.height,
        }
    }
}
