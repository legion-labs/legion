use ash::vk::{ExportSemaphoreCreateInfo, ExternalSemaphoreHandleTypeFlags};

use crate::{DeviceContext, ExternalResourceHandle, Semaphore};

pub(crate) struct VulkanSemaphore {
    vk_semaphore: ash::vk::Semaphore,
    export_capable: bool,
}

impl VulkanSemaphore {
    pub fn new(device_context: &DeviceContext, export_capable: bool) -> Self {
        let mut create_info =
            ash::vk::SemaphoreCreateInfo::builder().flags(ash::vk::SemaphoreCreateFlags::empty());

        #[cfg(target_os = "windows")]
        let handle_type = ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32;

        #[cfg(target_os = "linux")]
        let handle_type = ExternalSemaphoreHandleTypeFlags::OPAQUE_FD;

        let mut export_create_info = ExportSemaphoreCreateInfo::default();
        if export_capable {
            export_create_info.handle_types |= handle_type;

            create_info.p_next = std::ptr::addr_of!(export_create_info).cast::<std::ffi::c_void>();
        };

        let vk_semaphore = unsafe {
            device_context
                .vk_device()
                .create_semaphore(&*create_info, None)
                .unwrap()
        };

        Self {
            vk_semaphore,
            export_capable,
        }
    }

    pub fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_semaphore(self.vk_semaphore, None);
        }
    }

    pub fn external_semaphore_handle(
        &self,
        device_context: &DeviceContext,
    ) -> ExternalResourceHandle {
        assert!(self.export_capable);
        device_context.vk_external_semaphore_handle(self.vk_semaphore)
    }
}

impl Semaphore {
    pub fn vk_semaphore(&self) -> ash::vk::Semaphore {
        self.inner.backend_semaphore.vk_semaphore
    }

    pub fn vk_semaphore_ref(&self) -> &ash::vk::Semaphore {
        &self.inner.backend_semaphore.vk_semaphore
    }
}
