#![allow(unsafe_code)]

use std::sync::{Arc, Mutex};

use super::EncoderConfig;
use crate::{CpuBuffer, VideoProcessor};

mod cuda;
mod loader;

use ash::extensions::khr::{ExternalMemoryWin32, ExternalSemaphoreWin32};
pub use cuda::{CuContext, CuDevice};
use lgn_graphics_api::{DeviceContext, Semaphore, Texture};
pub use loader::{CudaApi, NvEncApi};
use nvenc_sys::{
    cuda::{
        CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1,
        CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1,
        CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1,
        CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1, CUarray,
        CUarray_format_enum, CUexternalMemory, CUexternalMemoryHandleType_enum,
        CUexternalSemaphore, CUexternalSemaphoreHandleType_enum, CUmipmappedArray, CUresult,
        CUDA_ARRAY3D_DESCRIPTOR, CUDA_EXTERNAL_MEMORY_HANDLE_DESC,
        CUDA_EXTERNAL_MEMORY_MIPMAPPED_ARRAY_DESC, CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC,
        CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS, CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS,
    },
    NVENCSTATUS, NVENC_INFINITE_GOPLENGTH, NV_ENCODE_API_FUNCTION_LIST,
    NV_ENCODE_API_FUNCTION_LIST_VER, NV_ENC_CODEC_H264_GUID, NV_ENC_CONFIG, NV_ENC_CONFIG_VER,
    NV_ENC_CREATE_BITSTREAM_BUFFER, NV_ENC_CREATE_BITSTREAM_BUFFER_VER, NV_ENC_INITIALIZE_PARAMS,
    NV_ENC_INITIALIZE_PARAMS_VER, NV_ENC_INPUT_PTR, NV_ENC_LOCK_BITSTREAM,
    NV_ENC_LOCK_BITSTREAM_VER, NV_ENC_MAP_INPUT_RESOURCE, NV_ENC_MAP_INPUT_RESOURCE_VER,
    NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS, NV_ENC_OUTPUT_PTR, NV_ENC_PIC_PARAMS,
    NV_ENC_PIC_PARAMS_VER, NV_ENC_PRESET_CONFIG, NV_ENC_PRESET_CONFIG_VER, NV_ENC_PRESET_P3_GUID,
    NV_ENC_REGISTERED_PTR, NV_ENC_REGISTER_RESOURCE, NV_ENC_REGISTER_RESOURCE_VER,
    NV_ENC_TUNING_INFO, _NV_ENC_BUFFER_FORMAT, _NV_ENC_BUFFER_USAGE, _NV_ENC_INPUT_RESOURCE_TYPE,
    _NV_ENC_PIC_STRUCT,
};

/// Nvenc Encoder Config
pub struct NvEncEncoderConfig {
    device_context: DeviceContext,
    width: u32,
    height: u32,
}

pub struct NvEncEncoderInner {
    nvenc: NvEncApi,
    encoder: *mut ::std::os::raw::c_void,
    cuda_context: CuContext,

    #[cfg(target_os = "windows")]
    external_memory: ExternalMemoryWin32,
    #[cfg(target_os = "linux")]
    external_memory: ExternalMemoryFd,
    cuda_image_memory: CUexternalMemory,
    cuda_mip_map_array: CUmipmappedArray,
    cuda_registered_image: NV_ENC_REGISTERED_PTR,

    cuda_mapped_images: [NV_ENC_INPUT_PTR; 3],
    cuda_bitstream_buffers: [NV_ENC_OUTPUT_PTR; 3],
    sent_frame: usize,
    received_frame: usize,

    encoder_width: u32,
    encoder_height: u32,

    #[cfg(target_os = "windows")]
    external_semaphore: ExternalSemaphoreWin32,
    #[cfg(target_os = "linux")]
    external_semaphore: ExternalSemaphoreFd,
    cuda_semaphore: CUexternalSemaphore,
}

pub struct NvEncEncoder {
    inner: Arc<Mutex<NvEncEncoderInner>>,
}

impl NvEncEncoder {
    pub fn encode_frame(&self) {
        self.wait_on_cuda_semaphore();

        let inner = &mut *self.inner.lock().unwrap();

        let mut nvenc_function_list = NV_ENCODE_API_FUNCTION_LIST {
            version: NV_ENCODE_API_FUNCTION_LIST_VER,
            ..NV_ENCODE_API_FUNCTION_LIST::default()
        };

        let mut result =
            unsafe { (inner.nvenc.create_instance)(std::ptr::addr_of_mut!(nvenc_function_list)) };

        assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

        let mut map_input_resource = NV_ENC_MAP_INPUT_RESOURCE {
            version: NV_ENC_MAP_INPUT_RESOURCE_VER,
            registeredResource: inner.cuda_registered_image,
            ..NV_ENC_MAP_INPUT_RESOURCE::default()
        };

        result = unsafe {
            (nvenc_function_list.nvEncMapInputResource.unwrap())(
                inner.encoder,
                std::ptr::addr_of_mut!(map_input_resource),
            )
        };
        assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

        inner.cuda_mapped_images[inner.sent_frame % 3] = map_input_resource.mappedResource;

        let mut pic_params = NV_ENC_PIC_PARAMS {
            version: NV_ENC_PIC_PARAMS_VER,
            pictureStruct: _NV_ENC_PIC_STRUCT::NV_ENC_PIC_STRUCT_FRAME,
            inputBuffer: inner.cuda_mapped_images[inner.sent_frame % 3],
            bufferFmt: _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ABGR,
            inputWidth: inner.encoder_width,
            inputHeight: inner.encoder_height,
            outputBitstream: inner.cuda_bitstream_buffers[inner.sent_frame % 3],
            ..NV_ENC_PIC_PARAMS::default()
        };
        inner.sent_frame += 1;

        result = unsafe {
            (nvenc_function_list.nvEncEncodePicture.unwrap())(
                inner.encoder,
                std::ptr::addr_of_mut!(pic_params),
            )
        };

        if result == NVENCSTATUS::NV_ENC_SUCCESS {
            while inner.received_frame < inner.sent_frame {
                let mut lock_bitstream_data = NV_ENC_LOCK_BITSTREAM {
                    version: NV_ENC_LOCK_BITSTREAM_VER,
                    outputBitstream: inner.cuda_bitstream_buffers[inner.received_frame % 3],
                    ..NV_ENC_LOCK_BITSTREAM::default()
                };
                lock_bitstream_data.set_doNotWait(0);

                result = unsafe {
                    (nvenc_function_list.nvEncLockBitstream.unwrap())(
                        inner.encoder,
                        std::ptr::addr_of_mut!(lock_bitstream_data),
                    )
                };
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                let data = unsafe {
                    std::slice::from_raw_parts(
                        lock_bitstream_data.bitstreamBufferPtr.cast::<*const u8>(),
                        lock_bitstream_data.bitstreamSizeInBytes as usize,
                    )
                };

                result = unsafe {
                    (nvenc_function_list.nvEncUnlockBitstream.unwrap())(
                        inner.encoder,
                        lock_bitstream_data.outputBitstream,
                    )
                };
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                if !inner.cuda_mapped_images[inner.received_frame % 3].is_null() {
                    result = unsafe {
                        (nvenc_function_list.nvEncUnmapInputResource.unwrap())(
                            inner.encoder,
                            inner.cuda_mapped_images[inner.received_frame % 3],
                        )
                    };
                    assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                    inner.cuda_mapped_images[inner.received_frame % 3] = std::ptr::null_mut();
                }
                inner.received_frame += 1;
            }
        }
    }

    pub fn cuda_image_from_vulkan(&self, image: &Texture) {
        self.destroy_cuda_image();

        let inner = &mut *self.inner.lock().unwrap();

        let create_info = ash::vk::MemoryGetWin32HandleInfoKHR {
            #[cfg(target_os = "windows")]
            handle_type: ash::vk::ExternalMemoryHandleTypeFlags::OPAQUE_WIN32,
            #[cfg(target_os = "linux")]
            handle_type: ash::vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD,
            memory: image.vk_device_memory(),
            ..ash::vk::MemoryGetWin32HandleInfoKHR::default()
        };

        let vulkan_image_memory = unsafe {
            inner
                .external_memory
                .get_memory_win32_handle(&create_info)
                .unwrap()
        };

        let handle = CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1 {
            win32: CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1 {
                handle: vulkan_image_memory,
                name: std::ptr::null_mut(),
            },
        };

        let memory_handle_desc = CUDA_EXTERNAL_MEMORY_HANDLE_DESC {
            #[cfg(target_os = "windows")]
            type_: CUexternalMemoryHandleType_enum::CU_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32,
            #[cfg(target_os = "linux")]
            type_: CUexternalMemoryHandleType_enum::CU_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD,
            handle,
            size: image.vk_alloc_size() as u64,
            ..CUDA_EXTERNAL_MEMORY_HANDLE_DESC::default()
        };

        inner.cuda_context.push();

        let mut result = unsafe {
            (inner.cuda_context.cuda_api().import_external_memory)(
                std::ptr::addr_of_mut!(inner.cuda_image_memory),
                std::ptr::addr_of!(memory_handle_desc),
            )
        };
        assert!(result == CUresult::CUDA_SUCCESS);

        let array_desc = CUDA_ARRAY3D_DESCRIPTOR {
            Width: image.extents().width as usize,
            Height: image.extents().height as usize,
            Depth: 0, /* CUDA 2D arrays are defined to have depth 0 */
            Format: CUarray_format_enum::CU_AD_FORMAT_UNSIGNED_INT8,
            NumChannels: 4,
            Flags: 0x22, //CUDA_ARRAY3D_SURFACE_LDST | CUDA_ARRAY3D_COLOR_ATTACHMENT,
        };

        let mipmap_array_desc = CUDA_EXTERNAL_MEMORY_MIPMAPPED_ARRAY_DESC {
            arrayDesc: array_desc,
            numLevels: 1,
            ..CUDA_EXTERNAL_MEMORY_MIPMAPPED_ARRAY_DESC::default()
        };

        result = unsafe {
            (inner
                .cuda_context
                .cuda_api()
                .external_memory_get_mapped_mipmapped_array)(
                std::ptr::addr_of_mut!(inner.cuda_mip_map_array),
                inner.cuda_image_memory,
                std::ptr::addr_of!(mipmap_array_desc),
            )
        };
        assert!(result == CUresult::CUDA_SUCCESS);

        let mut array: CUarray = std::ptr::null_mut();
        result = unsafe {
            (inner.cuda_context.cuda_api().mipmapped_array_get_level)(
                std::ptr::addr_of_mut!(array),
                inner.cuda_mip_map_array,
                0,
            )
        };
        assert!(result == CUresult::CUDA_SUCCESS);
        inner.cuda_context.pop();

        let mut register_resource = NV_ENC_REGISTER_RESOURCE {
            version: NV_ENC_REGISTER_RESOURCE_VER,
            resourceType: _NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_CUDAARRAY,
            width: image.extents().width,
            height: image.extents().height,
            pitch: image.extents().width * 4,
            resourceToRegister: array.cast::<std::ffi::c_void>(),
            bufferFormat: _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ABGR,
            bufferUsage: _NV_ENC_BUFFER_USAGE::NV_ENC_INPUT_IMAGE,
            ..NV_ENC_REGISTER_RESOURCE::default()
        };

        let mut nvenc_function_list = NV_ENCODE_API_FUNCTION_LIST {
            version: NV_ENCODE_API_FUNCTION_LIST_VER,
            ..NV_ENCODE_API_FUNCTION_LIST::default()
        };

        let mut result =
            unsafe { (inner.nvenc.create_instance)(std::ptr::addr_of_mut!(nvenc_function_list)) };

        assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

        result = unsafe {
            (nvenc_function_list.nvEncRegisterResource.unwrap())(
                inner.encoder,
                std::ptr::addr_of_mut!(register_resource),
            )
        };
        assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

        inner.cuda_registered_image = register_resource.registeredResource;
    }

    pub fn cuda_semaphore_from_vulkan(&self, semaphore: &Semaphore) {
        self.destroy_cuda_semaphore();

        let inner = &mut *self.inner.lock().unwrap();

        let create_info = ash::vk::SemaphoreGetWin32HandleInfoKHR {
            #[cfg(target_os = "windows")]
            handle_type: ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32,
            #[cfg(target_os = "linux")]
            handle_type: ash::vk::ExternalSemaphoreHandleTypeFlags::OPAQUE_FD,
            semaphore: semaphore.vk_semaphore(),
            ..ash::vk::SemaphoreGetWin32HandleInfoKHR::default()
        };

        let vulkan_semaphore = unsafe {
            inner
                .external_semaphore
                .get_semaphore_win32_handle(&create_info)
                .unwrap()
        };

        let handle = CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1 {
            win32: CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1 {
                handle: vulkan_semaphore,
                name: std::ptr::null_mut(),
            },
        };

        let sema_desc = CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC {
            #[cfg(target_os = "windows")]
            type_:
                CUexternalSemaphoreHandleType_enum::CU_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_WIN32,
            #[cfg(target_os = "linux")]
            type_: CUexternalSemaphoreHandleType_enum::CU_EXTERNAL_SEMAPHORE_HANDLE_TYPE_OPAQUE_FD,
            handle,
            ..CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC::default()
        };

        inner.cuda_context.push();
        let result = unsafe {
            (inner.cuda_context.cuda_api().import_external_semaphore)(
                std::ptr::addr_of_mut!(inner.cuda_semaphore),
                std::ptr::addr_of!(sema_desc),
            )
        };
        assert!(result == CUresult::CUDA_SUCCESS);
        inner.cuda_context.pop();
    }

    pub fn signal_cuda_semaphore(&self) {
        let inner = &mut *self.inner.lock().unwrap();

        let signal_params = CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS::default();

        unsafe {
            (inner
                .cuda_context
                .cuda_api()
                .signal_external_semaphores_async)(
                std::ptr::addr_of!(inner.cuda_semaphore),
                std::ptr::addr_of!(signal_params),
                1,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn wait_on_cuda_semaphore(&self) {
        let inner = &mut *self.inner.lock().unwrap();

        let wait_params = CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS::default();

        unsafe {
            (inner.cuda_context.cuda_api().wait_external_semaphores_async)(
                std::ptr::addr_of!(inner.cuda_semaphore),
                std::ptr::addr_of!(wait_params),
                1,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn destroy_cuda_semaphore(&self) {
        let inner = &mut *self.inner.lock().unwrap();
        unsafe {
            if !inner.cuda_semaphore.is_null() {
                (inner.cuda_context.cuda_api().destroy_external_semaphore)(inner.cuda_semaphore);
                inner.cuda_semaphore = std::ptr::null_mut();
            }
        }
    }

    fn destroy_cuda_image(&self) {
        let inner = &mut *self.inner.lock().unwrap();

        unsafe {
            let mut nvenc_function_list = NV_ENCODE_API_FUNCTION_LIST {
                version: NV_ENCODE_API_FUNCTION_LIST_VER,
                ..NV_ENCODE_API_FUNCTION_LIST::default()
            };

            let result = (inner.nvenc.create_instance)(std::ptr::addr_of_mut!(nvenc_function_list));
            assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

            if !inner.cuda_image_memory.is_null() {
                (inner.cuda_context.cuda_api().destroy_external_memory)(inner.cuda_image_memory);
                inner.cuda_image_memory = std::ptr::null_mut();
            }
            if !inner.cuda_mip_map_array.is_null() {
                (inner.cuda_context.cuda_api().mipmapped_array_destroy)(inner.cuda_mip_map_array);
                inner.cuda_mip_map_array = std::ptr::null_mut();
            }
            if !inner.cuda_registered_image.is_null() {
                (nvenc_function_list.nvEncUnregisterResource.unwrap())(
                    inner.encoder,
                    inner.cuda_registered_image,
                );
                inner.cuda_registered_image = std::ptr::null_mut();
            }
        }
    }

    fn destroy_cuda(&self) {
        let inner = &mut *self.inner.lock().unwrap();

        unsafe {
            let mut nvenc_function_list = NV_ENCODE_API_FUNCTION_LIST {
                version: NV_ENCODE_API_FUNCTION_LIST_VER,
                ..NV_ENCODE_API_FUNCTION_LIST::default()
            };

            let result = (inner.nvenc.create_instance)(std::ptr::addr_of_mut!(nvenc_function_list));

            assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

            for bitstream_buffer in &mut inner.cuda_bitstream_buffers {
                if !bitstream_buffer.is_null() {
                    (nvenc_function_list.nvEncDestroyBitstreamBuffer.unwrap())(
                        inner.encoder,
                        *bitstream_buffer,
                    );
                    *bitstream_buffer = std::ptr::null_mut();
                }
            }
        }
    }
}

impl Drop for NvEncEncoder {
    fn drop(&mut self) {
        self.destroy_cuda_image();
        self.destroy_cuda_semaphore();
        self.destroy_cuda();
    }
}

#[allow(unsafe_code)]
unsafe impl Send for NvEncEncoder {}
#[allow(unsafe_code)]
unsafe impl Sync for NvEncEncoder {}

impl VideoProcessor for NvEncEncoder {
    type Input = Texture;
    type Output = CpuBuffer;
    type Config = NvEncEncoderConfig;

    fn submit_input(&self, _image: &Self::Input) -> Result<(), crate::Error> {
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
                let mut nvenc_function_list = NV_ENCODE_API_FUNCTION_LIST {
                    version: NV_ENCODE_API_FUNCTION_LIST_VER,
                    ..NV_ENCODE_API_FUNCTION_LIST::default()
                };

                let mut result =
                    unsafe { (nvenc.create_instance)(std::ptr::addr_of_mut!(nvenc_function_list)) };
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                let mut open_session_ex_params = NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS {
                    version: nvenc_sys::NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER,
                    deviceType: nvenc_sys::NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
                    device: (*context.cuda_context()).cast::<std::ffi::c_void>(),
                    apiVersion: nvenc_sys::NVENCAPI_VERSION,
                    ..NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS::default()
                };

                let mut encoder: *mut ::std::os::raw::c_void = std::ptr::null_mut();
                result = unsafe {
                    (nvenc_function_list.nvEncOpenEncodeSessionEx.unwrap())(
                        std::ptr::addr_of_mut!(open_session_ex_params),
                        std::ptr::addr_of_mut!(encoder),
                    )
                };
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                let mut present_config = NV_ENC_PRESET_CONFIG {
                    version: NV_ENC_PRESET_CONFIG_VER,
                    presetCfg: NV_ENC_CONFIG {
                        version: NV_ENC_CONFIG_VER,
                        ..NV_ENC_CONFIG::default()
                    },
                    ..NV_ENC_PRESET_CONFIG::default()
                };

                result = unsafe {
                    (nvenc_function_list.nvEncGetEncodePresetConfigEx.unwrap())(
                        encoder,
                        NV_ENC_CODEC_H264_GUID,
                        NV_ENC_PRESET_P3_GUID,
                        NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
                        std::ptr::addr_of_mut!(present_config),
                    )
                };
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                present_config
                    .presetCfg
                    .encodeCodecConfig
                    .h264Config
                    .idrPeriod = NVENC_INFINITE_GOPLENGTH;

                let mut create_encode_params = NV_ENC_INITIALIZE_PARAMS {
                    version: NV_ENC_INITIALIZE_PARAMS_VER,
                    encodeGUID: NV_ENC_CODEC_H264_GUID,
                    presetGUID: NV_ENC_PRESET_P3_GUID,
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
                    tuningInfo: NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
                    ..NV_ENC_INITIALIZE_PARAMS::default()
                };
                result = unsafe {
                    (nvenc_function_list.nvEncInitializeEncoder.unwrap())(
                        encoder,
                        std::ptr::addr_of_mut!(create_encode_params),
                    )
                };
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                let mut cuda_bitstream_buffers = [std::ptr::null_mut(); 3];
                for cuda_bitstream_buffer in &mut cuda_bitstream_buffers {
                    let mut create_bitstream_buffer = NV_ENC_CREATE_BITSTREAM_BUFFER {
                        version: NV_ENC_CREATE_BITSTREAM_BUFFER_VER,
                        ..NV_ENC_CREATE_BITSTREAM_BUFFER::default()
                    };

                    result = unsafe {
                        (nvenc_function_list.nvEncCreateBitstreamBuffer.unwrap())(
                            encoder,
                            std::ptr::addr_of_mut!(create_bitstream_buffer),
                        )
                    };
                    assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);
                    *cuda_bitstream_buffer = create_bitstream_buffer.bitstreamBuffer;
                }

                return Some(Self {
                    inner: Arc::new(Mutex::new(NvEncEncoderInner {
                        nvenc,
                        encoder,
                        cuda_context: context,
                        external_memory: ExternalMemoryWin32::new(
                            config.device_context.vk_instance(),
                            config.device_context.vk_device(),
                        ),
                        cuda_image_memory: std::ptr::null_mut(),
                        cuda_mip_map_array: std::ptr::null_mut(),
                        cuda_registered_image: std::ptr::null_mut(),
                        cuda_mapped_images: [std::ptr::null_mut(); 3],
                        cuda_bitstream_buffers,
                        encoder_width: config.width,
                        encoder_height: config.height,
                        sent_frame: 0,
                        received_frame: 0,
                        external_semaphore: ExternalSemaphoreWin32::new(
                            config.device_context.vk_instance(),
                            config.device_context.vk_device(),
                        ),
                        cuda_semaphore: std::ptr::null_mut(),
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
            device_context: config.gfx_config.clone(),
            width: config.width,
            height: config.height,
        }
    }
}
