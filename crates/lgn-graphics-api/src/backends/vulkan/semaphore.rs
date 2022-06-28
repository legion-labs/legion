use ash::vk::{ExportSemaphoreCreateInfo, ExternalSemaphoreHandleTypeFlags};

use crate::{DeviceContext, ExternalResourceHandle, Semaphore, SemaphoreDef, SemaphoreUsage};

pub(crate) struct VulkanSemaphore {
    vk_semaphore: ash::vk::Semaphore,
}

impl VulkanSemaphore {
    pub fn new(device_context: &DeviceContext, semaphore_def: SemaphoreDef) -> Self {
        let mut create_info =
            ash::vk::SemaphoreCreateInfo::builder().flags(ash::vk::SemaphoreCreateFlags::empty());

        let mut export_create_info = ExportSemaphoreCreateInfo::builder();
        if semaphore_def.usage_flags.intersects(SemaphoreUsage::EXPORT) {
            #[cfg(target_os = "windows")]
            let handle_type = ExternalSemaphoreHandleTypeFlags::OPAQUE_WIN32;

            #[cfg(target_os = "linux")]
            let handle_type = ExternalSemaphoreHandleTypeFlags::OPAQUE_FD;

            export_create_info = export_create_info.handle_types(handle_type);
            create_info = create_info.push_next(&mut export_create_info);
        };

        let mut type_create_info = ash::vk::SemaphoreTypeCreateInfo::builder();
        if semaphore_def
            .usage_flags
            .intersects(SemaphoreUsage::TIMELINE)
        {
            type_create_info = type_create_info
                .semaphore_type(ash::vk::SemaphoreType::TIMELINE)
                .initial_value(semaphore_def.initial_value);

            create_info = create_info.push_next(&mut type_create_info);
        }

        let vk_semaphore = unsafe {
            device_context
                .vk_device()
                .create_semaphore(&*create_info, None)
                .unwrap()
        };

        Self { vk_semaphore }
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
        device_context.vk_external_semaphore_handle(self.vk_semaphore)
    }
}

impl Semaphore {
    pub fn vk_semaphore(&self) -> ash::vk::Semaphore {
        self.inner.backend_semaphore.vk_semaphore
    }

    pub fn backend_timeline_value(&self) -> u64 {
        unsafe {
            self.inner
                .device_context
                .vk_device()
                .get_semaphore_counter_value(self.vk_semaphore())
                .unwrap()
        }
    }
}
