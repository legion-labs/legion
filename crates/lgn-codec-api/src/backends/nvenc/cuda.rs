#![allow(unsafe_code)]

use std::sync::Arc;

use nvenc_sys::cuda::{cudaError_enum, CUcontext, CUdevice};

use super::loader::CudaApi;

#[derive(Clone)]
pub struct CuDevice {
    inner: Arc<CuDeviceInner>,
}

struct CuDeviceInner {
    device: CUdevice,
    cuda_api: CudaApi,
}

impl CuDevice {
    pub fn new(cuda_api: CudaApi) -> Option<Self> {
        let result = unsafe { (cuda_api.init)(0) };
        assert_eq!(result, cudaError_enum::CUDA_SUCCESS);
        let mut num_devices = 0;
        let result = unsafe { (cuda_api.device_get_count)(&mut num_devices) };
        assert_eq!(result, cudaError_enum::CUDA_SUCCESS);

        if num_devices != 0 {
            let mut dev: CUdevice = CUdevice::default();
            let result = unsafe { (cuda_api.device_get)(&mut dev, 0) };
            assert_eq!(result, cudaError_enum::CUDA_SUCCESS);

            Some(Self {
                inner: Arc::new(CuDeviceInner {
                    device: dev,
                    cuda_api,
                }),
            })
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct CuContext {
    inner: Arc<CuContextInner>,
}

impl CuContext {
    pub fn new(device: &CuDevice) -> Option<Self> {
        let mut context: CUcontext = std::ptr::null_mut();
        let result =
            unsafe { (device.inner.cuda_api.ctx_create)(&mut context, 0, device.inner.device) };
        if result == cudaError_enum::CUDA_SUCCESS {
            Some(Self {
                inner: Arc::new(CuContextInner {
                    context,
                    device: device.clone(),
                }),
            })
        } else {
            None
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
    }

    pub(crate) fn cuda_context(&self) -> &CUcontext {
        &self.inner.context
    }

    pub(crate) fn cuda_api(&self) -> &CudaApi {
        &self.inner.device.inner.cuda_api
    }
}

struct CuContextInner {
    device: CuDevice,
    context: CUcontext,
}
