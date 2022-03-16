#![allow(unsafe_code)]

use std::sync::Arc;

use nvenc_sys::cuda::{cudaError_enum, CUcontext, CUdevice, CUuuid};

use super::loader::CudaApi;
use crate::{Error, Result};

#[derive(Clone)]
pub struct CuDevice {
    inner: Arc<CuDeviceInner>,
}

struct CuDeviceInner {
    device: CUdevice,
    cuda_api: CudaApi,
}

impl CuDevice {
    pub fn new(cuda_api: CudaApi, guid: &str) -> Result<Self> {
        let result = unsafe { (cuda_api.init)(0) };
        assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
        let mut num_devices = 0;
        let result = unsafe { (cuda_api.device_get_count)(&mut num_devices) };
        assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
        let guid = uuid::Uuid::parse_str(guid).unwrap();
        for i in 0..num_devices {
            let mut dev: CUdevice = CUdevice::default();
            let result = unsafe { (cuda_api.device_get)(&mut dev, i) };
            assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
            let mut id = CUuuid::default();
            let result = unsafe { (cuda_api.device_get_uuid)(&mut id, dev) };
            assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
            let id_u8 = unsafe {
                std::slice::from_raw_parts(id.bytes.as_ptr().cast::<u8>(), id.bytes.len())
            };
            if id_u8 == guid.as_bytes() {
                return Ok(Self {
                    inner: Arc::new(CuDeviceInner {
                        device: dev,
                        cuda_api,
                    }),
                });
            }
        }
        Err(Error::Init {
            encoder: "nvenc",
            reason: "device not found".to_string(),
        })
    }
}

#[derive(Clone)]
pub struct CuContext {
    inner: Arc<CuContextInner>,
}

impl CuContext {
    pub fn new(device: &CuDevice) -> Result<Self> {
        let mut context: CUcontext = std::ptr::null_mut();
        let result =
            unsafe { (device.inner.cuda_api.ctx_create)(&mut context, 0, device.inner.device) };
        if result == cudaError_enum::CUDA_SUCCESS {
            Ok(Self {
                inner: Arc::new(CuContextInner {
                    context,
                    device: device.clone(),
                }),
            })
        } else {
            Err(Error::Init {
                encoder: "nvenc",
                reason: "coundn't create a cuda context".to_string(),
            })
        }
    }

    pub fn push(&self) {
        let result =
            unsafe { (self.inner.device.inner.cuda_api.ctx_push_current)(self.inner.context) };
        assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
    }

    pub fn pop(&self) {
        let mut context: CUcontext = std::ptr::null_mut();
        let result = unsafe { (self.inner.device.inner.cuda_api.ctx_pop_current)(&mut context) };
        assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
        // we should have one context per thread ? is it thread safe ? to be tested
        assert_eq!(context, std::ptr::null_mut());
    }
}

struct CuContextInner {
    device: CuDevice,
    context: CUcontext,
}
