#![allow(unsafe_code)]

use std::ffi::CStr;

use lgn_graphics_api::{Extents3D, Texture};
use lgn_tracing::{error, span_fn};

use nvenc_sys::{
    NVENCSTATUS, NVENC_INFINITE_GOPLENGTH, NV_ENCODE_API_FUNCTION_LIST, NV_ENC_CODEC_H264_GUID,
    NV_ENC_CONFIG, NV_ENC_CONFIG_VER, NV_ENC_CREATE_BITSTREAM_BUFFER,
    NV_ENC_CREATE_BITSTREAM_BUFFER_VER, NV_ENC_INITIALIZE_PARAMS, NV_ENC_INITIALIZE_PARAMS_VER,
    NV_ENC_INPUT_PTR, NV_ENC_LOCK_BITSTREAM, NV_ENC_LOCK_BITSTREAM_VER, NV_ENC_MAP_INPUT_RESOURCE,
    NV_ENC_MAP_INPUT_RESOURCE_VER, NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS, NV_ENC_OUTPUT_PTR,
    NV_ENC_PIC_PARAMS, NV_ENC_PIC_PARAMS_VER, NV_ENC_PRESET_CONFIG, NV_ENC_PRESET_CONFIG_VER,
    NV_ENC_PRESET_P3_GUID, NV_ENC_RECONFIGURE_PARAMS, NV_ENC_RECONFIGURE_PARAMS_VER,
    NV_ENC_REGISTERED_PTR, NV_ENC_REGISTER_RESOURCE, NV_ENC_REGISTER_RESOURCE_VER,
    NV_ENC_TUNING_INFO, _NV_ENC_BUFFER_FORMAT, _NV_ENC_BUFFER_USAGE, _NV_ENC_INPUT_RESOURCE_TYPE,
    _NV_ENC_PIC_STRUCT,
};

use crate::{encoder_resource::EncoderResource, stream_encoder::EncoderWorkItem};

use super::nv_encoder::NvEncoder;

pub(crate) struct NvEncoderSession {
    nv_encoder: NvEncoder,
    hw_encoder: *mut ::std::os::raw::c_void,
    function_list: NV_ENCODE_API_FUNCTION_LIST,

    encoder_width: u32,
    encoder_height: u32,

    registered_resource: NV_ENC_REGISTERED_PTR,
    mapped_resource: NV_ENC_INPUT_PTR,
    cuda_bitstream_buffer: NV_ENC_OUTPUT_PTR,
}

unsafe impl Send for NvEncoderSession {}
unsafe impl Sync for NvEncoderSession {}

impl Drop for NvEncoderSession {
    fn drop(&mut self) {
        unsafe {
            (self.function_list.nvEncDestroyBitstreamBuffer.unwrap())(
                self.hw_encoder,
                self.cuda_bitstream_buffer,
            );
            (self.function_list.nvEncDestroyEncoder.unwrap())(self.hw_encoder);
        }
    }
}

impl NvEncoderSession {
    pub fn new(nv_encoder: &NvEncoder) -> Option<Self> {
        let mut open_session_ex_params = NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS {
            version: nvenc_sys::NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER,
            deviceType: nvenc_sys::NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
            device: nv_encoder
                .context()
                .cuda_context()
                .cast::<std::ffi::c_void>(),
            apiVersion: nvenc_sys::NVENCAPI_VERSION,
            ..NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS::default()
        };

        let function_list = nv_encoder.function_list();
        let mut hw_encoder = std::ptr::null_mut();

        let result = unsafe {
            (function_list.nvEncOpenEncodeSessionEx.unwrap())(
                std::ptr::addr_of_mut!(open_session_ex_params),
                std::ptr::addr_of_mut!(hw_encoder),
            )
        };
        if result == NVENCSTATUS::NV_ENC_SUCCESS {
            Some(Self {
                nv_encoder: nv_encoder.clone(),
                function_list,
                hw_encoder,
                encoder_width: 0,
                encoder_height: 0,
                registered_resource: std::ptr::null_mut(),
                mapped_resource: std::ptr::null_mut(),
                cuda_bitstream_buffer: std::ptr::null_mut(),
            })
        } else {
            error!("Error opening encoder session {:?}", result);
            None
        }
    }

    fn get_encode_params(
        &mut self,
        image_etents: &Extents3D,
        resize: bool,
    ) -> NV_ENC_INITIALIZE_PARAMS {
        let mut present_config = NV_ENC_PRESET_CONFIG {
            version: NV_ENC_PRESET_CONFIG_VER,
            presetCfg: NV_ENC_CONFIG {
                version: NV_ENC_CONFIG_VER,
                ..NV_ENC_CONFIG::default()
            },
            ..NV_ENC_PRESET_CONFIG::default()
        };

        let encode_config = if resize {
            std::ptr::null_mut()
        } else {
            let result = unsafe {
                self.function_list.nvEncGetEncodePresetConfigEx.map(|func| {
                    (func)(
                        self.hw_encoder,
                        NV_ENC_CODEC_H264_GUID,
                        NV_ENC_PRESET_P3_GUID,
                        NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
                        std::ptr::addr_of_mut!(present_config),
                    )
                })
            };
            if result.unwrap() != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error retreving encoder config {:?}",
                        CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                            self.hw_encoder
                        ))
                    );
                }
            }

            present_config
                .presetCfg
                .encodeCodecConfig
                .h264Config
                .idrPeriod = NVENC_INFINITE_GOPLENGTH;

            std::ptr::addr_of_mut!(present_config.presetCfg)
        };

        self.encoder_width = image_etents.width;
        self.encoder_height = image_etents.height;

        NV_ENC_INITIALIZE_PARAMS {
            version: NV_ENC_INITIALIZE_PARAMS_VER,
            encodeGUID: NV_ENC_CODEC_H264_GUID,
            presetGUID: NV_ENC_PRESET_P3_GUID,
            encodeWidth: self.encoder_width,
            encodeHeight: self.encoder_height,
            darWidth: self.encoder_width,
            darHeight: self.encoder_height,
            frameRateNum: 60,
            frameRateDen: 1,
            enableEncodeAsync: 0,
            enablePTD: 1,
            encodeConfig: encode_config,
            maxEncodeWidth: 3840,
            maxEncodeHeight: 2160,
            tuningInfo: NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
            ..NV_ENC_INITIALIZE_PARAMS::default()
        }
    }

    #[span_fn]
    pub fn configure_cuda_encoder(&mut self, image: &EncoderResource<Texture>) {
        let external_image = image.external_resource();

        if self.encoder_width == 0 || self.encoder_height == 0 {
            let mut create_encode_params = self.get_encode_params(external_image.extents(), false);

            let mut result = unsafe {
                (self.function_list.nvEncInitializeEncoder.unwrap())(
                    self.hw_encoder,
                    std::ptr::addr_of_mut!(create_encode_params),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error initializing encoder {:?}",
                        CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                            self.hw_encoder
                        ))
                    );
                }
            }

            let mut create_bitstream_buffer = NV_ENC_CREATE_BITSTREAM_BUFFER {
                version: NV_ENC_CREATE_BITSTREAM_BUFFER_VER,
                ..NV_ENC_CREATE_BITSTREAM_BUFFER::default()
            };

            result = unsafe {
                (self.function_list.nvEncCreateBitstreamBuffer.unwrap())(
                    self.hw_encoder,
                    std::ptr::addr_of_mut!(create_bitstream_buffer),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error creating output bit streams encoder {:?}",
                        CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                            self.hw_encoder
                        ))
                    );
                }
            }
            self.cuda_bitstream_buffer = create_bitstream_buffer.bitstreamBuffer;
        } else if self.encoder_width != external_image.extents().width
            || self.encoder_height != external_image.extents().height
        {
            let mut reconfig_encode_params = NV_ENC_RECONFIGURE_PARAMS {
                version: NV_ENC_RECONFIGURE_PARAMS_VER,
                reInitEncodeParams: self.get_encode_params(external_image.extents(), true),
                ..NV_ENC_RECONFIGURE_PARAMS::default()
            };

            let result = unsafe {
                (self.function_list.nvEncReconfigureEncoder.unwrap())(
                    self.hw_encoder,
                    std::ptr::addr_of_mut!(reconfig_encode_params),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error resizing stream {:?}. Width {}, Height {}",
                        CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                            self.hw_encoder
                        )),
                        external_image.extents().width,
                        external_image.extents().height
                    );
                }
            }

            self.encoder_width = external_image.extents().width;
            self.encoder_height = external_image.extents().height;
        }
    }

    #[span_fn]
    pub(crate) fn encode_frame(&mut self, input: &EncoderWorkItem) {
        self.configure_cuda_encoder(&input.image);

        self.nv_encoder
            .wait_on_external_semaphore(input.semaphore.internal_resource());

        let array = self
            .nv_encoder
            .image_from_key(input.image.internal_resource());

        let mut register_resource = NV_ENC_REGISTER_RESOURCE {
            version: NV_ENC_REGISTER_RESOURCE_VER,
            resourceType: _NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_CUDAARRAY,
            width: self.encoder_width,
            height: self.encoder_height,
            pitch: self.encoder_width * 4,
            resourceToRegister: array.cast::<std::ffi::c_void>(),
            bufferFormat: _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ARGB,
            bufferUsage: _NV_ENC_BUFFER_USAGE::NV_ENC_INPUT_IMAGE,
            ..NV_ENC_REGISTER_RESOURCE::default()
        };

        let result = unsafe {
            (self.function_list.nvEncRegisterResource.unwrap())(
                self.hw_encoder,
                std::ptr::addr_of_mut!(register_resource),
            )
        };
        if result != NVENCSTATUS::NV_ENC_SUCCESS {
            unsafe {
                let message = CStr::from_ptr(
                    (self.function_list.nvEncGetLastErrorString.unwrap())(self.hw_encoder),
                );
                error!("Error registering encoder resource {:?}", message);
            }
        }
        self.registered_resource = register_resource.registeredResource;

        let mut map_input_resource = NV_ENC_MAP_INPUT_RESOURCE {
            version: NV_ENC_MAP_INPUT_RESOURCE_VER,
            registeredResource: self.registered_resource,
            ..NV_ENC_MAP_INPUT_RESOURCE::default()
        };

        let result = unsafe {
            (self.function_list.nvEncMapInputResource.unwrap())(
                self.hw_encoder,
                std::ptr::addr_of_mut!(map_input_resource),
            )
        };
        if result != NVENCSTATUS::NV_ENC_SUCCESS {
            unsafe {
                let message = CStr::from_ptr(
                    (self.function_list.nvEncGetLastErrorString.unwrap())(self.hw_encoder),
                );
                error!("Error mapping encoder input buffer {:?}", message);
            }
        }
        self.mapped_resource = map_input_resource.mappedResource;

        let mut pic_params = NV_ENC_PIC_PARAMS {
            version: NV_ENC_PIC_PARAMS_VER,
            pictureStruct: _NV_ENC_PIC_STRUCT::NV_ENC_PIC_STRUCT_FRAME,
            inputBuffer: self.mapped_resource,
            bufferFmt: _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ARGB,
            inputWidth: self.encoder_width,
            inputHeight: self.encoder_height,
            outputBitstream: self.cuda_bitstream_buffer,
            ..NV_ENC_PIC_PARAMS::default()
        };

        let result = unsafe {
            (self.function_list.nvEncEncodePicture.unwrap())(
                self.hw_encoder,
                std::ptr::addr_of_mut!(pic_params),
            )
        };
        if result != NVENCSTATUS::NV_ENC_SUCCESS {
            unsafe {
                error!(
                    "Error encoding picture {:?}",
                    CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                        self.hw_encoder
                    ))
                );
            }
        }
    }

    #[span_fn]
    pub fn process_encoded_data(&mut self) -> Vec<u8> {
        let lock_bitstream_data = {
            let mut lock_bitstream_data = NV_ENC_LOCK_BITSTREAM {
                version: NV_ENC_LOCK_BITSTREAM_VER,
                outputBitstream: self.cuda_bitstream_buffer,
                ..NV_ENC_LOCK_BITSTREAM::default()
            };
            lock_bitstream_data.set_doNotWait(0);

            let result = unsafe {
                (self.function_list.nvEncLockBitstream.unwrap())(
                    self.hw_encoder,
                    std::ptr::addr_of_mut!(lock_bitstream_data),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error locking bitstream buffer {:?}",
                        CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                            self.hw_encoder
                        ))
                    );
                }
            }
            lock_bitstream_data
        };

        let data = unsafe {
            std::slice::from_raw_parts(
                lock_bitstream_data.bitstreamBufferPtr.cast::<u8>(),
                lock_bitstream_data.bitstreamSizeInBytes as usize,
            )
        };

        let output = Vec::<u8>::from(data);

        {
            let result = unsafe {
                (self.function_list.nvEncUnlockBitstream.unwrap())(
                    self.hw_encoder,
                    lock_bitstream_data.outputBitstream,
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error unlocking bitstream buffer {:?}",
                        CStr::from_ptr((self.function_list.nvEncGetLastErrorString.unwrap())(
                            self.hw_encoder
                        ))
                    );
                }
            }

            let mut result = unsafe {
                (self.function_list.nvEncUnmapInputResource.unwrap())(
                    self.hw_encoder,
                    self.mapped_resource,
                )
            };
            self.mapped_resource = std::ptr::null_mut();
            assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

            result = unsafe {
                (self.function_list.nvEncUnregisterResource.unwrap())(
                    self.hw_encoder,
                    self.registered_resource,
                )
            };
            self.registered_resource = std::ptr::null_mut();
            assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);
        }
        output
    }
}
