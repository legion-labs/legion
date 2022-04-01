#![allow(unsafe_code)]

use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc, Mutex},
};

use lgn_graphics_api::{DeviceContext, ExternalResource, Semaphore, Texture};

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
        CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC, CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS,
    },
    NVENCSTATUS, NV_ENCODE_API_FUNCTION_LIST, NV_ENCODE_API_FUNCTION_LIST_VER,
};

use super::{CuContext, CuDevice, CudaApi, NvEncApi};

static NEXT_IMAGE_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
static NEXT_SEMAPHORE_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub struct NvEncoderInner {
    _nvenc: NvEncApi,
    context: CuContext,
    function_list: NV_ENCODE_API_FUNCTION_LIST,

    cuda_semaphore_map: HashMap<u64, CUexternalSemaphore>,
    cuda_image_map: HashMap<u64, (CUexternalMemory, CUmipmappedArray, CUarray)>,
}

#[derive(Clone)]
pub struct NvEncoder {
    inner: Arc<Mutex<NvEncoderInner>>,
}

unsafe impl Send for NvEncoder {}
unsafe impl Sync for NvEncoder {}

impl NvEncoder {
    pub(crate) fn new() -> Option<Self> {
        if let Some(context) = CudaApi::load()
            .and_then(CuDevice::new)
            .and_then(|device| CuContext::new(&device))
        {
            if let Some(nvenc) = NvEncApi::load() {
                let mut function_list = NV_ENCODE_API_FUNCTION_LIST {
                    version: NV_ENCODE_API_FUNCTION_LIST_VER,
                    ..NV_ENCODE_API_FUNCTION_LIST::default()
                };

                let result =
                    unsafe { (nvenc.create_instance)(std::ptr::addr_of_mut!(function_list)) };
                if result == NVENCSTATUS::NV_ENC_SUCCESS {
                    return Some(Self {
                        inner: Arc::new(Mutex::new(NvEncoderInner {
                            _nvenc: nvenc,
                            context,
                            function_list,
                            cuda_semaphore_map: HashMap::new(),
                            cuda_image_map: HashMap::new(),
                        })),
                    });
                }
            }
        }
        None
    }

    pub(crate) fn register_external_image(
        &self,
        device_context: &DeviceContext,
        external_image: &Texture,
    ) -> u64 {
        let handle = CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1 {
            #[cfg(target_os = "windows")]
            win32: CUDA_EXTERNAL_MEMORY_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1 {
                handle: external_image.external_resource_handle(device_context),
                name: std::ptr::null_mut(),
            },
            #[cfg(target_os = "linux")]
            fd: external_image.external_resource_handle(device_context),
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

        let inner = &mut *self.inner.lock().unwrap();
        inner.context.push();

        let mut cuda_image_memory = std::ptr::null_mut();
        let result = unsafe {
            (inner.context.cuda_api().import_external_memory)(
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
        let result = unsafe {
            (inner
                .context
                .cuda_api()
                .external_memory_get_mapped_mipmapped_array)(
                std::ptr::addr_of_mut!(cuda_mip_map_array),
                cuda_image_memory,
                std::ptr::addr_of!(mipmap_array_desc),
            )
        };
        assert_eq!(result, CUresult::CUDA_SUCCESS);

        let mut array: CUarray = std::ptr::null_mut();
        let result = unsafe {
            (inner.context.cuda_api().mipmapped_array_get_level)(
                std::ptr::addr_of_mut!(array),
                cuda_mip_map_array,
                0,
            )
        };
        assert_eq!(result, CUresult::CUDA_SUCCESS);
        inner.context.pop();

        let new_image_data = (cuda_image_memory, cuda_mip_map_array, array);

        let new_key: u64 = NEXT_IMAGE_KEY.fetch_add(1, Ordering::Relaxed);

        inner.cuda_image_map.insert(new_key, new_image_data);
        new_key
    }

    pub(crate) fn image_from_key(&self, image_key: u64) -> CUarray {
        let inner = &mut *self.inner.lock().unwrap();

        inner.cuda_image_map.get(&image_key).unwrap().2
    }

    pub(crate) fn unregister_external_image(&self, image_key: u64) {
        let inner = &mut *self.inner.lock().unwrap();

        if let Some((cuda_image_memory, cuda_mip_map_array, _)) =
            inner.cuda_image_map.remove(&image_key)
        {
            inner.context.push();
            unsafe {
                let result = (inner.context.cuda_api().mipmapped_array_destroy)(cuda_mip_map_array);
                assert_eq!(result, CUresult::CUDA_SUCCESS);

                let result = (inner.context.cuda_api().destroy_external_memory)(cuda_image_memory);
                assert_eq!(result, CUresult::CUDA_SUCCESS);
            }
            inner.context.pop();
        }
    }

    pub(crate) fn register_external_semaphore(
        &self,
        device_context: &DeviceContext,
        external_semaphore: &Semaphore,
    ) -> u64 {
        let handle = CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1 {
            #[cfg(target_os = "windows")]
            win32: CUDA_EXTERNAL_SEMAPHORE_HANDLE_DESC_st__bindgen_ty_1__bindgen_ty_1 {
                handle: external_semaphore.external_resource_handle(device_context),
                name: std::ptr::null_mut(),
            },
            #[cfg(target_os = "linux")]
            fd: external_semaphore.external_resource_handle(device_context),
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

        let inner = &mut *self.inner.lock().unwrap();
        inner.context.push();
        let mut cuda_semaphore = std::ptr::null_mut();
        let result = unsafe {
            (inner.context.cuda_api().import_external_semaphore)(
                std::ptr::addr_of_mut!(cuda_semaphore),
                std::ptr::addr_of!(sema_desc),
            )
        };
        assert_eq!(result, CUresult::CUDA_SUCCESS);
        inner.context.pop();

        let new_key: u64 = NEXT_SEMAPHORE_KEY.fetch_add(1, Ordering::Relaxed);

        inner.cuda_semaphore_map.insert(new_key, cuda_semaphore);
        new_key
    }

    pub(crate) fn wait_on_external_semaphore(&self, semaphore_key: u64) {
        let inner = &mut *self.inner.lock().unwrap();

        let cuda_semaphore = *inner.cuda_semaphore_map.get(&semaphore_key).unwrap();

        let wait_params = CUDA_EXTERNAL_SEMAPHORE_WAIT_PARAMS::default();

        inner.context.push();
        let result = unsafe {
            (inner.context.cuda_api().wait_external_semaphores_async)(
                std::ptr::addr_of!(cuda_semaphore),
                std::ptr::addr_of!(wait_params),
                1,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(result, CUresult::CUDA_SUCCESS);
        inner.context.pop();
    }

    pub(crate) fn unregister_external_semaphore(&self, semaphore_key: u64) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.context.push();
        if let Some(cuda_semaphore) = inner.cuda_semaphore_map.remove(&semaphore_key) {
            let result =
                unsafe { (inner.context.cuda_api().destroy_external_semaphore)(cuda_semaphore) };
            assert_eq!(result, CUresult::CUDA_SUCCESS);
        }
        inner.context.pop();
    }

    pub(crate) fn context(&self) -> CuContext {
        let inner = self.inner.lock().unwrap();
        inner.context.clone()
    }

    pub(crate) fn function_list(&self) -> NV_ENCODE_API_FUNCTION_LIST {
        let inner = self.inner.lock().unwrap();
        inner.function_list
    }
}
