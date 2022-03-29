#![allow(unsafe_code)]

use std::{
    collections::HashMap,
    ffi::CStr,
    sync::{atomic::Ordering, Arc, Mutex},
};

use lgn_graphics_api::{Extents3D, Semaphore, Texture};
use lgn_tracing::{error, span_fn, span_scope};

#[cfg(target_os = "windows")]
use nvenc_sys::cuda::{
    CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1,
    CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1,
};

use nvenc_sys::{
    cuda::{
        CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1,
        CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1, CUarray, CUarray_format_enum,
        CUexternalMemory, CUexternalMemoryHandleType_enum, CUexternalSemaphore,
        CUexternalSemaphoreHandleType_enum, CUmipmappedArray, CUresult, CUDA_ARRAY3D_DESCRIPTOR,
        CUDA_EXTERNAL_MEMORY_HANDLE_DESC, CUDA_EXTERNAL_MEMORY_MIPMAPPED_ARRAY_DESC,
        CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC, CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS,
        CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS,
    },
    NVENCSTATUS, NVENC_INFINITE_GOPLENGTH, NV_ENCODE_API_FUNCTION_LIST,
    NV_ENCODE_API_FUNCTION_LIST_VER, NV_ENC_CODEC_H264_GUID, NV_ENC_CONFIG, NV_ENC_CONFIG_VER,
    NV_ENC_CREATE_BITSTREAM_BUFFER, NV_ENC_CREATE_BITSTREAM_BUFFER_VER, NV_ENC_INITIALIZE_PARAMS,
    NV_ENC_INITIALIZE_PARAMS_VER, NV_ENC_INPUT_PTR, NV_ENC_LOCK_BITSTREAM,
    NV_ENC_LOCK_BITSTREAM_VER, NV_ENC_MAP_INPUT_RESOURCE, NV_ENC_MAP_INPUT_RESOURCE_VER,
    NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS, NV_ENC_OUTPUT_PTR, NV_ENC_PIC_PARAMS,
    NV_ENC_PIC_PARAMS_VER, NV_ENC_PRESET_CONFIG, NV_ENC_PRESET_CONFIG_VER, NV_ENC_PRESET_P3_GUID,
    NV_ENC_RECONFIGURE_PARAMS, NV_ENC_RECONFIGURE_PARAMS_VER, NV_ENC_REGISTER_RESOURCE,
    NV_ENC_REGISTER_RESOURCE_VER, NV_ENC_TUNING_INFO, _NV_ENC_BUFFER_FORMAT, _NV_ENC_BUFFER_USAGE,
    _NV_ENC_INPUT_RESOURCE_TYPE, _NV_ENC_PIC_STRUCT,
};

use crate::{
    encoder_resource::EncoderResource,
    encoder_work_queue::{EncoderWorkItem, EncoderWorkQueue},
};

use super::{CuContext, CuDevice, CudaApi, NvEncApi};

struct NvEncEncoderInner {
    nvenc: NvEncApi,
    function_list: NV_ENCODE_API_FUNCTION_LIST,
    encoder: *mut ::std::os::raw::c_void,
    cuda_context: CuContext,

    encoder_width: u32,
    encoder_height: u32,
    cuda_bitstream_buffers: [NV_ENC_OUTPUT_PTR; 5],
    sent_frame: usize,
    received_frame: usize,

    current_cuda_semaphore_key: u64,
    cuda_semaphore_map: HashMap<u64, CUexternalSemaphore>,

    cuda_image_map: HashMap<
        u64,
        (
            CUexternalMemory,
            CUmipmappedArray,
            *mut ::std::os::raw::c_void,
            NV_ENC_INPUT_PTR,
        ),
    >,
}

impl NvEncEncoderInner {
    fn destroy_cuda_encoder(&mut self) {
        unsafe {
            for bit_stream_buffer in &mut self.cuda_bitstream_buffers {
                (self.function_list.nvEncDestroyBitstreamBuffer.unwrap())(
                    self.encoder,
                    *bit_stream_buffer,
                );
            }
            (self.function_list.nvEncDestroyEncoder.unwrap())(self.encoder);
        }
    }
}

impl Drop for NvEncEncoderInner {
    fn drop(&mut self) {
        self.destroy_cuda_encoder();
    }
}

#[derive(Clone)]
pub struct NvEncEncoder {
    inner: Arc<Mutex<NvEncEncoderInner>>,
}

unsafe impl Send for NvEncEncoder {}
unsafe impl Sync for NvEncEncoder {}

static NEXT_IMAGE_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl NvEncEncoder {
    pub(crate) fn encoder_loop(work_queue: &mut EncoderWorkQueue, encoder: &Self) {
        while !work_queue.shutting_down() {
            if let Some(semaphore_key) = work_queue.internal_semaphore_for_cleanup() {
                encoder.destroy_cuda_semaphore(semaphore_key);
            }

            if let Some(semaphore_key) = work_queue.internal_image_for_cleanup() {
                encoder.destroy_cuda_image(semaphore_key);
            }
        }
    }

    pub(crate) fn new() -> Option<Self> {
        if let Some(context) = CudaApi::load()
            .and_then(CuDevice::new)
            .and_then(|device| CuContext::new(&device))
        {
            if let Some(nvenc) = NvEncApi::load() {
                return Some(Self {
                    inner: Arc::new(Mutex::new(NvEncEncoderInner {
                        nvenc,
                        function_list: NV_ENCODE_API_FUNCTION_LIST {
                            version: NV_ENCODE_API_FUNCTION_LIST_VER,
                            ..NV_ENCODE_API_FUNCTION_LIST::default()
                        },
                        encoder: std::ptr::null_mut(),
                        cuda_context: context,
                        encoder_width: 0,
                        encoder_height: 0,
                        cuda_bitstream_buffers: [std::ptr::null_mut(); 5],
                        sent_frame: 0,
                        received_frame: 0,
                        current_cuda_semaphore_key: 0,
                        cuda_semaphore_map: HashMap::new(),
                        cuda_image_map: HashMap::new(),
                    })),
                });
            }
        }
        None
    }

    pub fn initialize_encoder(&self) {
        let inner = &mut *self.inner.lock().unwrap();

        let mut result =
            unsafe { (inner.nvenc.create_instance)(std::ptr::addr_of_mut!(inner.function_list)) };
        if result != NVENCSTATUS::NV_ENC_SUCCESS {
            unsafe {
                error!(
                    "Error creating encoder instance {:?}",
                    CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                        inner.encoder
                    ))
                );
            }
        }

        let mut open_session_ex_params = NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS {
            version: nvenc_sys::NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER,
            deviceType: nvenc_sys::NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
            device: (*inner.cuda_context.cuda_context()).cast::<std::ffi::c_void>(),
            apiVersion: nvenc_sys::NVENCAPI_VERSION,
            ..NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS::default()
        };

        result = unsafe {
            (inner.function_list.nvEncOpenEncodeSessionEx.unwrap())(
                std::ptr::addr_of_mut!(open_session_ex_params),
                std::ptr::addr_of_mut!(inner.encoder),
            )
        };
        if result != NVENCSTATUS::NV_ENC_SUCCESS {
            unsafe {
                error!(
                    "Error opening encoder session {:?}",
                    CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                        inner.encoder
                    ))
                );
            }
        }
    }

    fn get_encode_params(
        inner: &mut NvEncEncoderInner,
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
                inner
                    .function_list
                    .nvEncGetEncodePresetConfigEx
                    .map(|func| {
                        (func)(
                            inner.encoder,
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
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
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

        inner.encoder_width = image_etents.width;
        inner.encoder_height = image_etents.height;

        NV_ENC_INITIALIZE_PARAMS {
            version: NV_ENC_INITIALIZE_PARAMS_VER,
            encodeGUID: NV_ENC_CODEC_H264_GUID,
            presetGUID: NV_ENC_PRESET_P3_GUID,
            encodeWidth: inner.encoder_width,
            encodeHeight: inner.encoder_height,
            darWidth: inner.encoder_width,
            darHeight: inner.encoder_height,
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
    pub fn configure_cuda_encoder(
        &self,
        image: &EncoderResource<Texture>,
    ) -> (
        CUexternalMemory,
        CUmipmappedArray,
        *mut ::std::os::raw::c_void,
        NV_ENC_INPUT_PTR,
    ) {
        let inner = &mut *self.inner.lock().unwrap();
        let external_image = image.external_resource();

        if inner.encoder_width == 0 || inner.encoder_height == 0 {
            let mut create_encode_params =
                Self::get_encode_params(inner, external_image.extents(), false);

            let mut result = unsafe {
                (inner.function_list.nvEncInitializeEncoder.unwrap())(
                    inner.encoder,
                    std::ptr::addr_of_mut!(create_encode_params),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error initializing encoder {:?}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
                        ))
                    );
                }
            }

            for bit_stream_buffer in &mut inner.cuda_bitstream_buffers {
                let mut create_bitstream_buffer = NV_ENC_CREATE_BITSTREAM_BUFFER {
                    version: NV_ENC_CREATE_BITSTREAM_BUFFER_VER,
                    ..NV_ENC_CREATE_BITSTREAM_BUFFER::default()
                };

                result = unsafe {
                    (inner.function_list.nvEncCreateBitstreamBuffer.unwrap())(
                        inner.encoder,
                        std::ptr::addr_of_mut!(create_bitstream_buffer),
                    )
                };
                if result != NVENCSTATUS::NV_ENC_SUCCESS {
                    unsafe {
                        error!(
                            "Error creating output bit streams encoder {:?}",
                            CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                                inner.encoder
                            ))
                        );
                    }
                }
                *bit_stream_buffer = create_bitstream_buffer.bitstreamBuffer;
            }
        } else if inner.encoder_width != external_image.extents().width
            || inner.encoder_height != external_image.extents().height
        {
            let mut reconfig_encode_params = NV_ENC_RECONFIGURE_PARAMS {
                version: NV_ENC_RECONFIGURE_PARAMS_VER,
                reInitEncodeParams: Self::get_encode_params(inner, external_image.extents(), true),
                ..NV_ENC_RECONFIGURE_PARAMS::default()
            };

            let result = unsafe {
                (inner.function_list.nvEncReconfigureEncoder.unwrap())(
                    inner.encoder,
                    std::ptr::addr_of_mut!(reconfig_encode_params),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error resizing stream {:?}. Width {}, Height {}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
                        )),
                        external_image.extents().width,
                        external_image.extents().height
                    );
                }
            }

            inner.encoder_width = external_image.extents().width;
            inner.encoder_height = external_image.extents().height;
        }

        if image.internal_resource() == u64::MAX {
            let handle = CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1 {
                #[cfg(target_os = "windows")]
                win32: CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1 {
                    handle: image.external_resource_handle(),
                    name: std::ptr::null_mut(),
                },
                #[cfg(target_os = "linux")]
                fd: image.external_resource_handle(),
            };

            let memory_handle_desc = CUDA_EXTERNAL_MEMORY_HANDLE_DESC {
                #[cfg(target_os = "windows")]
                type_: CUexternalMemoryHandleType_enum::CU_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_WIN32,
                #[cfg(target_os = "linux")]
                type_: CUexternalMemoryHandleType_enum::CU_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD,
                handle,
                size: external_image.vk_alloc_size() as u64,
                ..CUDA_EXTERNAL_MEMORY_HANDLE_DESC::default()
            };

            inner.cuda_context.push();

            let mut cuda_image_memory = std::ptr::null_mut();
            let mut result = unsafe {
                (inner.cuda_context.cuda_api().import_external_memory)(
                    std::ptr::addr_of_mut!(cuda_image_memory),
                    std::ptr::addr_of!(memory_handle_desc),
                )
            };
            assert!(result == CUresult::CUDA_SUCCESS);

            let array_desc = CUDA_ARRAY3D_DESCRIPTOR {
                Width: external_image.extents().width as usize,
                Height: external_image.extents().height as usize,
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

            let mut cuda_mip_map_array = std::ptr::null_mut();
            result = unsafe {
                (inner
                    .cuda_context
                    .cuda_api()
                    .external_memory_get_mapped_mipmapped_array)(
                    std::ptr::addr_of_mut!(cuda_mip_map_array),
                    cuda_image_memory,
                    std::ptr::addr_of!(mipmap_array_desc),
                )
            };
            assert!(result == CUresult::CUDA_SUCCESS);

            let mut array: CUarray = std::ptr::null_mut();
            result = unsafe {
                (inner.cuda_context.cuda_api().mipmapped_array_get_level)(
                    std::ptr::addr_of_mut!(array),
                    cuda_mip_map_array,
                    0,
                )
            };
            assert!(result == CUresult::CUDA_SUCCESS);

            let mut register_resource = NV_ENC_REGISTER_RESOURCE {
                version: NV_ENC_REGISTER_RESOURCE_VER,
                resourceType: _NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_CUDAARRAY,
                width: inner.encoder_width,
                height: inner.encoder_height,
                pitch: inner.encoder_width * 4,
                resourceToRegister: array.cast::<std::ffi::c_void>(),
                bufferFormat: _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ABGR,
                bufferUsage: _NV_ENC_BUFFER_USAGE::NV_ENC_INPUT_IMAGE,
                ..NV_ENC_REGISTER_RESOURCE::default()
            };

            let result = unsafe {
                (inner.function_list.nvEncRegisterResource.unwrap())(
                    inner.encoder,
                    std::ptr::addr_of_mut!(register_resource),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error registering encoder resource {:?}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
                        ))
                    );
                }
            }

            inner.cuda_context.pop();

            let mut map_input_resource = NV_ENC_MAP_INPUT_RESOURCE {
                version: NV_ENC_MAP_INPUT_RESOURCE_VER,
                registeredResource: register_resource.registeredResource,
                ..NV_ENC_MAP_INPUT_RESOURCE::default()
            };

            let result = unsafe {
                (inner.function_list.nvEncMapInputResource.unwrap())(
                    inner.encoder,
                    std::ptr::addr_of_mut!(map_input_resource),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error mapping encoder input buffer {:?}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
                        ))
                    );
                }
            }

            let new_image_data = (
                cuda_image_memory,
                cuda_mip_map_array,
                register_resource.registeredResource,
                map_input_resource.mappedResource,
            );

            let new_key: u64 = NEXT_IMAGE_KEY.fetch_add(1, Ordering::Relaxed);

            image.update_internal_resource(new_key);
            inner.cuda_image_map.insert(new_key, new_image_data);
            new_image_data
        } else {
            *inner
                .cuda_image_map
                .get(&image.internal_resource())
                .unwrap()
        }
    }

    #[span_fn]
    pub fn cuda_semaphore_from_encoder(
        &self,
        semaphore: &EncoderResource<Semaphore>,
    ) -> CUexternalSemaphore {
        let inner = &mut *self.inner.lock().unwrap();

        if semaphore.internal_resource() == u64::MAX {
            let handle = CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1 {
                #[cfg(target_os = "windows")]
                win32: CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1 {
                    handle: semaphore.external_resource_handle(),
                    name: std::ptr::null_mut(),
                },
                #[cfg(target_os = "linux")]
                fd: semaphore.external_resource_handle(),
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
            let mut cuda_semaphore = std::ptr::null_mut();
            let result = unsafe {
                (inner.cuda_context.cuda_api().import_external_semaphore)(
                    std::ptr::addr_of_mut!(cuda_semaphore),
                    std::ptr::addr_of!(sema_desc),
                )
            };
            assert!(result == CUresult::CUDA_SUCCESS);
            inner.cuda_context.pop();

            let new_key: u64 = inner.current_cuda_semaphore_key;
            inner.current_cuda_semaphore_key += 1;

            semaphore.update_internal_resource(new_key);
            inner.cuda_semaphore_map.insert(new_key, cuda_semaphore);
            cuda_semaphore
        } else {
            *inner
                .cuda_semaphore_map
                .get(&semaphore.internal_resource())
                .unwrap()
        }
    }

    fn backlogged(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.sent_frame - inner.received_frame > 4
    }

    #[span_fn]
    pub(crate) fn encode_frame(&self, input: &EncoderWorkItem) {
        while self.backlogged() {
            span_scope!("waiting");
        }

        self.wait_on_encoder_semaphore(&input.semaphore);

        let (_, _, _, cuda_mapped_buffer) = self.configure_cuda_encoder(&input.image);

        {
            let inner = &mut *self.inner.lock().unwrap();

            let mut pic_params = NV_ENC_PIC_PARAMS {
                version: NV_ENC_PIC_PARAMS_VER,
                pictureStruct: _NV_ENC_PIC_STRUCT::NV_ENC_PIC_STRUCT_FRAME,
                inputBuffer: cuda_mapped_buffer,
                bufferFmt: _NV_ENC_BUFFER_FORMAT::NV_ENC_BUFFER_FORMAT_ABGR,
                inputWidth: inner.encoder_width,
                inputHeight: inner.encoder_height,
                outputBitstream: inner.cuda_bitstream_buffers[inner.sent_frame % 5],
                ..NV_ENC_PIC_PARAMS::default()
            };
            inner.sent_frame += 1;

            let result = unsafe {
                (inner.function_list.nvEncEncodePicture.unwrap())(
                    inner.encoder,
                    std::ptr::addr_of_mut!(pic_params),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error encoding picture {:?}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
                        ))
                    );
                }
            }
        }
    }

    #[span_fn]
    pub fn process_encoded_data(&self) -> Vec<u8> {
        let lock_bitstream_data = {
            let inner = self.inner.lock().unwrap();
            let mut lock_bitstream_data = NV_ENC_LOCK_BITSTREAM {
                version: NV_ENC_LOCK_BITSTREAM_VER,
                outputBitstream: inner.cuda_bitstream_buffers[inner.received_frame % 5],
                ..NV_ENC_LOCK_BITSTREAM::default()
            };
            lock_bitstream_data.set_doNotWait(0);

            let result = unsafe {
                (inner.function_list.nvEncLockBitstream.unwrap())(
                    inner.encoder,
                    std::ptr::addr_of_mut!(lock_bitstream_data),
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error locking bitstream buffer {:?}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
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
            let inner = &mut *self.inner.lock().unwrap();
            let result = unsafe {
                (inner.function_list.nvEncUnlockBitstream.unwrap())(
                    inner.encoder,
                    lock_bitstream_data.outputBitstream,
                )
            };
            if result != NVENCSTATUS::NV_ENC_SUCCESS {
                unsafe {
                    error!(
                        "Error unlocking bitstream buffer {:?}",
                        CStr::from_ptr((inner.function_list.nvEncGetLastErrorString.unwrap())(
                            inner.encoder
                        ))
                    );
                }
            }
            inner.received_frame += 1;
        }
        output
    }

    #[span_fn]
    pub fn signal_encoder_semaphore(&mut self, semaphore: &EncoderResource<Semaphore>) {
        let inner = &mut *self.inner.lock().unwrap();

        let cuda_semaphore = self.cuda_semaphore_from_encoder(semaphore);
        let signal_params = CUDA_EXTERNAL_SEMAPHORE_SIGNAL_PARAMS::default();

        inner.cuda_context.push();
        unsafe {
            (inner
                .cuda_context
                .cuda_api()
                .signal_external_semaphores_async)(
                std::ptr::addr_of!(cuda_semaphore),
                std::ptr::addr_of!(signal_params),
                1,
                std::ptr::null_mut(),
            );
        }
        inner.cuda_context.pop();
    }

    #[span_fn]
    pub fn wait_on_encoder_semaphore(&self, semaphore: &EncoderResource<Semaphore>) {
        let cuda_semaphore = self.cuda_semaphore_from_encoder(semaphore);

        let inner = &mut *self.inner.lock().unwrap();
        let wait_params = CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS::default();

        inner.cuda_context.push();
        unsafe {
            (inner.cuda_context.cuda_api().wait_external_semaphores_async)(
                std::ptr::addr_of!(cuda_semaphore),
                std::ptr::addr_of!(wait_params),
                1,
                std::ptr::null_mut(),
            );
        }
        inner.cuda_context.pop();
    }

    pub fn destroy_cuda_semaphore(&self, semaphore_key: u64) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.cuda_context.push();
        if let Some(cuda_semaphore) = inner.cuda_semaphore_map.remove(&semaphore_key) {
            unsafe {
                (inner.cuda_context.cuda_api().destroy_external_semaphore)(cuda_semaphore);
            }
        }
        inner.cuda_context.pop();
    }

    fn destroy_cuda_image(&self, image_key: u64) {
        let inner = &mut *self.inner.lock().unwrap();

        if let Some((
            cuda_image_memory,
            cuda_mip_map_array,
            cuda_registered_image,
            cuda_mapped_buffer,
        )) = inner.cuda_image_map.remove(&image_key)
        {
            inner.cuda_context.push();
            unsafe {
                let mut result = (inner.function_list.nvEncUnmapInputResource.unwrap())(
                    inner.encoder,
                    cuda_mapped_buffer,
                );
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                result = (inner.function_list.nvEncUnregisterResource.unwrap())(
                    inner.encoder,
                    cuda_registered_image,
                );
                assert!(result == NVENCSTATUS::NV_ENC_SUCCESS);

                let mut result =
                    (inner.cuda_context.cuda_api().mipmapped_array_destroy)(cuda_mip_map_array);
                assert!(result == CUresult::CUDA_SUCCESS);

                result = (inner.cuda_context.cuda_api().destroy_external_memory)(cuda_image_memory);
                assert!(result == CUresult::CUDA_SUCCESS);
            }
            inner.cuda_context.pop();
        }
    }
}
