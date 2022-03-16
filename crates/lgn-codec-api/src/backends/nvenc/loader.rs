#![allow(unsafe_code)]

use libloading::Library;
use nvenc_sys::{
    cuda::{
        CuCtxCreateFn, CuCtxDestroyFn, CuCtxPopCurrentFn, CuCtxPushCurrentFn,
        CuDestroyExternalMemoryFn, CuDestroyExternalSemaphoreFn, CuDeviceGetCountFn, CuDeviceGetFn,
        CuDeviceGetNameFn, CuDeviceGetUuidFn, CuExternalMemoryGetMappedBufferFn,
        CuExternalMemoryGetMappedMipmappedArrayFn, CuGetErrorNameFn, CuGetErrorStringFn,
        CuImportExternalMemoryFn, CuImportExternalSemaphoreFn, CuInitFn, CuMemAllocHostFn,
        CuMemAllocPitchFn, CuMemFreeFn, CuMemcpy2DAsyncFn, CuMemcpy2DFn, CuMemcpy2DUnalignedFn,
        CuMemcpyDtoHFn, CuMipmappedArrayDestroyFn, CuMipmappedArrayGetLevelFn,
        CuSignalExternalSemaphoresAsyncFn, CuStreamCreateFn, CuStreamDestroyFn,
        CuWaitExternalSemaphoresAsyncFn, CUDA_DLL_NAME, CU_CTX_CREATE_FN_NAME,
        CU_CTX_DESTROY_FN_NAME, CU_CTX_POP_CURRENT_FN_NAME, CU_CTX_PUSH_CURRENT_FN_NAME,
        CU_DESTROY_EXTERNAL_MEMORY_FN_NAME, CU_DESTROY_EXTERNAL_SEMAPHORE_FN_NAME,
        CU_DEVICE_GET_COUNT_FN_NAME, CU_DEVICE_GET_FN_NAME, CU_DEVICE_GET_NAME_FN_NAME,
        CU_DEVICE_GET_UUID_FN_NAME, CU_EXTERNAL_MEMORY_GET_MAPPED_BUFFER_FN_NAME,
        CU_EXTERNAL_MEMORY_GET_MAPPED_MIPMAPPED_ARRAY_FN_NAME, CU_GET_ERROR_NAME_FN_NAME,
        CU_GET_ERROR_STRING_FN_NAME, CU_IMPORT_EXTERNAL_MEMORY_FN_NAME,
        CU_IMPORT_EXTERNAL_SEMAPHORE_FN_NAME, CU_INIT_FN_NAME, CU_MEMCPY_2D_ASYNC_FN_NAME,
        CU_MEMCPY_2D_FN_NAME, CU_MEMCPY_2D_UNALIGNED_FN_NAME, CU_MEMCPY_D_TO_H_FN_NAME,
        CU_MEM_ALLOC_HOST_FN_NAME, CU_MEM_ALLOC_PITCH_FN_NAME, CU_MEM_FREE_FN_NAME,
        CU_MEM_FREE_HOST_FN_NAME, CU_MIPMAPPED_ARRAY_DESTROY_FN_NAME,
        CU_MIPMAPPED_ARRAY_GET_LEVEL_FN_NAME, CU_SIGNAL_EXTERNAL_SEMAPHORES_ASYNC_FN_NAME,
        CU_STREAM_CREATE_FN_NAME, CU_STREAM_DESTROY_FN_NAME,
        CU_WAIT_EXTERNAL_SEMAPHORES_ASYNC_FN_NAME,
    },
    NvEncodeApiCreateInstanceFn, NvEncodeApiGetMaxSupportedVersionFn, NVENC_DLL_NAME,
    NV_ENCODE_API_CREATE_INSTANCE_FN_NAME, NV_ENCODE_API_GET_MAX_SUPPORTED_VERSION_FN_NAME,
};
use std::ops::Deref;

pub struct CudaApi {
    pub init: CuInitFn,
    pub get_error_string: CuGetErrorStringFn,
    pub get_error_name: CuGetErrorNameFn,
    pub device_get_count: CuDeviceGetCountFn,
    pub device_get: CuDeviceGetFn,
    pub device_get_name: CuDeviceGetNameFn,
    pub device_get_uuid: CuDeviceGetUuidFn,
    pub ctx_create: CuCtxCreateFn,
    pub ctx_destroy: CuCtxDestroyFn,
    pub ctx_push_current: CuCtxPushCurrentFn,
    pub ctx_pop_current: CuCtxPopCurrentFn,
    pub stream_create: CuStreamCreateFn,
    pub stream_destroy: CuStreamDestroyFn,
    pub mem_alloc_host: CuMemAllocHostFn,
    pub mem_alloc_pitch: CuMemAllocPitchFn,
    pub mem_free_fn: CuMemFreeFn,
    pub mem_free_host: CuMemAllocHostFn,
    pub memcpy_2d: CuMemcpy2DFn,
    pub memcpy_2d_unaligned: CuMemcpy2DUnalignedFn,
    pub memcpy_2d_async: CuMemcpy2DAsyncFn,
    pub memcpy_d_to_h: CuMemcpyDtoHFn,
    pub import_external_memory: CuImportExternalMemoryFn,
    pub import_external_semaphore: CuImportExternalSemaphoreFn,
    pub external_memory_get_mapped_buffer: CuExternalMemoryGetMappedBufferFn,
    pub external_memory_get_mapped_mipmapped_array: CuExternalMemoryGetMappedMipmappedArrayFn,
    pub mipmapped_array_get_level: CuMipmappedArrayGetLevelFn,
    pub mipmapped_array_destroy: CuMipmappedArrayDestroyFn,
    pub destroy_external_memory: CuDestroyExternalMemoryFn,
    pub destroy_external_semaphore: CuDestroyExternalSemaphoreFn,
    pub wait_external_semaphores_async: CuWaitExternalSemaphoresAsyncFn,
    pub signal_external_semaphores_async: CuSignalExternalSemaphoresAsyncFn,

    _dll: Library,
}

impl CudaApi {
    pub fn load() -> Option<Self> {
        if let Ok(dll) = unsafe { Library::new(CUDA_DLL_NAME) } {
            fn load_symbol<T: Copy>(dll: &Library, symbol: &[u8]) -> T {
                unsafe {
                    *dll.get::<T>(symbol)
                        .expect("failed to load cuda function")
                        .deref()
                }
            }
            let init = load_symbol::<CuInitFn>(&dll, CU_INIT_FN_NAME);
            let get_error_string =
                load_symbol::<CuGetErrorStringFn>(&dll, CU_GET_ERROR_STRING_FN_NAME);
            let get_error_name = load_symbol::<CuGetErrorNameFn>(&dll, CU_GET_ERROR_NAME_FN_NAME);
            let device_get_count =
                load_symbol::<CuDeviceGetCountFn>(&dll, CU_DEVICE_GET_COUNT_FN_NAME);
            let get_device = load_symbol::<CuDeviceGetFn>(&dll, CU_DEVICE_GET_FN_NAME);
            let device_get_name =
                load_symbol::<CuDeviceGetNameFn>(&dll, CU_DEVICE_GET_NAME_FN_NAME);
            let device_get_uuid =
                load_symbol::<CuDeviceGetUuidFn>(&dll, CU_DEVICE_GET_UUID_FN_NAME);
            let ctx_create = load_symbol::<CuCtxCreateFn>(&dll, CU_CTX_CREATE_FN_NAME);
            let ctx_destroy = load_symbol::<CuCtxDestroyFn>(&dll, CU_CTX_DESTROY_FN_NAME);
            let ctx_push_current =
                load_symbol::<CuCtxPushCurrentFn>(&dll, CU_CTX_PUSH_CURRENT_FN_NAME);
            let ctx_pop_current =
                load_symbol::<CuCtxPopCurrentFn>(&dll, CU_CTX_POP_CURRENT_FN_NAME);
            let stream_create = load_symbol::<CuStreamCreateFn>(&dll, CU_STREAM_CREATE_FN_NAME);
            let stream_destroy = load_symbol::<CuStreamDestroyFn>(&dll, CU_STREAM_DESTROY_FN_NAME);
            let mem_alloc_host = load_symbol::<CuMemAllocHostFn>(&dll, CU_MEM_ALLOC_HOST_FN_NAME);
            let mem_alloc_pitch =
                load_symbol::<CuMemAllocPitchFn>(&dll, CU_MEM_ALLOC_PITCH_FN_NAME);
            let mem_free_fn = load_symbol::<CuMemFreeFn>(&dll, CU_MEM_FREE_FN_NAME);
            let mem_free_host = load_symbol::<CuMemAllocHostFn>(&dll, CU_MEM_FREE_HOST_FN_NAME);
            let memcpy_2d = load_symbol::<CuMemcpy2DFn>(&dll, CU_MEMCPY_2D_FN_NAME);
            let memcpy_2d_unaligned =
                load_symbol::<CuMemcpy2DUnalignedFn>(&dll, CU_MEMCPY_2D_UNALIGNED_FN_NAME);
            let memcpy_2d_async =
                load_symbol::<CuMemcpy2DAsyncFn>(&dll, CU_MEMCPY_2D_ASYNC_FN_NAME);
            let memcpy_d_to_h = load_symbol::<CuMemcpyDtoHFn>(&dll, CU_MEMCPY_D_TO_H_FN_NAME);
            let import_external_memory =
                load_symbol::<CuImportExternalMemoryFn>(&dll, CU_IMPORT_EXTERNAL_MEMORY_FN_NAME);
            let import_external_semaphore = load_symbol::<CuImportExternalSemaphoreFn>(
                &dll,
                CU_IMPORT_EXTERNAL_SEMAPHORE_FN_NAME,
            );
            let external_memory_get_mapped_buffer = load_symbol::<CuExternalMemoryGetMappedBufferFn>(
                &dll,
                CU_EXTERNAL_MEMORY_GET_MAPPED_BUFFER_FN_NAME,
            );
            let external_memory_get_mapped_mipmapped_array =
                load_symbol::<CuExternalMemoryGetMappedMipmappedArrayFn>(
                    &dll,
                    CU_EXTERNAL_MEMORY_GET_MAPPED_MIPMAPPED_ARRAY_FN_NAME,
                );
            let mipmapped_array_get_level = load_symbol::<CuMipmappedArrayGetLevelFn>(
                &dll,
                CU_MIPMAPPED_ARRAY_GET_LEVEL_FN_NAME,
            );
            let mipmapped_array_destroy =
                load_symbol::<CuMipmappedArrayDestroyFn>(&dll, CU_MIPMAPPED_ARRAY_DESTROY_FN_NAME);
            let destroy_external_memory =
                load_symbol::<CuDestroyExternalMemoryFn>(&dll, CU_DESTROY_EXTERNAL_MEMORY_FN_NAME);
            let destroy_external_semaphore = load_symbol::<CuDestroyExternalSemaphoreFn>(
                &dll,
                CU_DESTROY_EXTERNAL_SEMAPHORE_FN_NAME,
            );
            let wait_external_semaphores_async = load_symbol::<CuWaitExternalSemaphoresAsyncFn>(
                &dll,
                CU_WAIT_EXTERNAL_SEMAPHORES_ASYNC_FN_NAME,
            );
            let signal_external_semaphores_async = load_symbol::<CuSignalExternalSemaphoresAsyncFn>(
                &dll,
                CU_SIGNAL_EXTERNAL_SEMAPHORES_ASYNC_FN_NAME,
            );

            Some(Self {
                init,
                get_error_string,
                get_error_name,
                device_get_count,
                device_get: get_device,
                device_get_name,
                device_get_uuid,
                ctx_create,
                ctx_destroy,
                ctx_push_current,
                ctx_pop_current,
                stream_create,
                stream_destroy,
                mem_alloc_host,
                mem_alloc_pitch,
                mem_free_fn,
                mem_free_host,
                memcpy_2d,
                memcpy_2d_unaligned,
                memcpy_2d_async,
                memcpy_d_to_h,
                import_external_memory,
                import_external_semaphore,
                external_memory_get_mapped_buffer,
                external_memory_get_mapped_mipmapped_array,
                mipmapped_array_get_level,
                mipmapped_array_destroy,
                destroy_external_memory,
                destroy_external_semaphore,
                wait_external_semaphores_async,
                signal_external_semaphores_async,
                _dll: dll,
            })
        } else {
            None
        }
    }
}

pub struct NvEncApi {
    pub get_max_supported_version: NvEncodeApiGetMaxSupportedVersionFn,
    pub create_instance: NvEncodeApiCreateInstanceFn,

    _dll: Library,
}

impl NvEncApi {
    pub fn load() -> Option<Self> {
        if let Ok(dll) = unsafe { Library::new(NVENC_DLL_NAME) } {
            fn load_symbol<T: Copy>(dll: &Library, symbol: &[u8]) -> T {
                unsafe {
                    *dll.get::<T>(symbol)
                        .expect("failed to load nvenc function")
                        .deref()
                }
            }
            let get_max_supported_version = load_symbol::<NvEncodeApiGetMaxSupportedVersionFn>(
                &dll,
                NV_ENCODE_API_GET_MAX_SUPPORTED_VERSION_FN_NAME,
            );
            let create_instance = load_symbol::<NvEncodeApiCreateInstanceFn>(
                &dll,
                NV_ENCODE_API_CREATE_INSTANCE_FN_NAME,
            );

            Some(Self {
                //get_last_error_string,
                get_max_supported_version,
                create_instance,
                _dll: dll,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_nvidia_cuda_api() {
        let cuda = CudaApi::load();
        assert!(cuda.is_some());
    }

    #[test]
    #[ignore]
    fn test_nvidia_nvenc_api() {
        let nvenc = NvEncApi::load();
        assert!(nvenc.is_some());
    }
}
