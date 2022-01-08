use std::ffi::{CStr, CString};
use std::sync::Arc;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::DebugUtilsMessageTypeFlagsEXT;
use lgn_tracing::{debug, error, info, log_enabled, trace, warn, Level};

use crate::backends::vulkan::check_extensions_availability;
use crate::backends::vulkan::VkDebugReporter;
use crate::{ExtensionMode, GfxError, GfxResult};

/// Create one of these at startup. It never gets lost/destroyed.
pub struct VkInstance {
    pub entry: Arc<ash::Entry>,
    pub instance: ash::Instance,
    pub debug_reporter: Option<VkDebugReporter>,
}

impl VkInstance {
    /// Creates a vulkan instance.
    pub fn new(
        entry: ash::Entry,
        app_name: &CString,
        validation_mode: ExtensionMode,
        windowing_mode: ExtensionMode,
    ) -> GfxResult<Self> {
        // Determine the supported version of vulkan that's available
        let vulkan_version = match entry.try_enumerate_instance_version()? {
            // Vulkan 1.1+
            Some(version) => version,
            // Vulkan 1.0
            None => vk::make_api_version(0, 1, 0, 0),
        };

        let vulkan_version_tuple = (
            vk::api_version_major(vulkan_version),
            vk::api_version_minor(vulkan_version),
            vk::api_version_patch(vulkan_version),
        );

        info!("Found Vulkan version: {:?}", vulkan_version_tuple);

        // Only need 1.1 for negative y viewport support, which is also possible to get
        // out of an extension, but at this point I think 1.1 is a reasonable
        // minimum expectation
        if vulkan_version < vk::API_VERSION_1_1 {
            return Err(GfxError::from(vk::Result::ERROR_INCOMPATIBLE_DRIVER));
        }

        // Expected to be 1.1.0 or 1.0.0 depending on what we found in
        // try_enumerate_instance_version https://vulkan.lunarg.com/doc/view/1.1.70.1/windows/tutorial/html/16-vulkan_1_1_changes.html

        // Info that's exposed to the driver. In a real shipped product, this data might
        // be used by the driver to make specific adjustments to improve
        // performance https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkApplicationInfo.html
        let appinfo = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .application_version(0)
            .engine_name(app_name)
            .engine_version(0)
            .api_version(vulkan_version);

        // Get the available layers/extensions
        let (requied_layer_names, requied_extension_names) =
            Self::find_layers_and_extensions(&entry, validation_mode, windowing_mode)?;

        if log_enabled!(Level::Debug) {
            debug!("Using layers: {:?}", requied_layer_names);
            debug!("Using extensions: {:?}", requied_extension_names);
        }

        let layer_names: Vec<_> = requied_layer_names.iter().map(|x| x.as_ptr()).collect();
        let extension_names: Vec<_> = requied_extension_names.iter().map(|x| x.as_ptr()).collect();

        // Create the instance
        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&appinfo)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names);

        info!("Creating vulkan instance");
        let instance: ash::Instance = unsafe { entry.create_instance(&create_info, None)? };

        // Setup the debug callback for the validation layer
        let debug_reporter = if requied_extension_names
            .iter()
            .any(|extension_name| *extension_name == DebugUtils::name())
        {
            Some(Self::setup_vulkan_debug_callback(&entry, &instance)?)
        } else {
            None
        };

        Ok(Self {
            entry: Arc::new(entry),
            instance,
            debug_reporter,
        })
    }

    /// This is used to setup a debug callback for logging validation errors
    fn setup_vulkan_debug_callback(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> VkResult<VkDebugReporter> {
        info!("Setting up vulkan debug callback");
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            )
            .message_type(
                DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(super::debug_reporter::vulkan_debug_callback));

        let debug_report_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let debug_callback =
            unsafe { debug_report_loader.create_debug_utils_messenger(&debug_info, None)? };

        Ok(VkDebugReporter {
            debug_report_loader,
            debug_callback,
        })
    }

    fn find_layers_and_extensions(
        entry: &ash::Entry,
        validation_mode: ExtensionMode,
        windowing_mode: ExtensionMode,
    ) -> GfxResult<(Vec<&CStr>, Vec<&CStr>)> {
        let layers = entry.enumerate_instance_layer_properties()?;
        debug!("Available layers: {:#?}", layers);
        let extensions = entry.enumerate_instance_extension_properties()?;
        debug!("Available extensions: {:#?}", extensions);
        let mut layer_names = vec![];
        let mut extension_names = vec![];
        info!("Validation mode: {:?}", validation_mode);
        match validation_mode {
            ExtensionMode::Disabled => {}
            ExtensionMode::EnabledIfAvailable | ExtensionMode::Enabled => {
                match (
                    Self::find_best_validation_layer(&layers),
                    check_extensions_availability(&[DebugUtils::name()], &extensions),
                ) {
                    (Some(validation_layer), true) => {
                        layer_names.push(validation_layer);
                        extension_names.push(DebugUtils::name());
                    }
                    (_, _) => {
                        if validation_mode == ExtensionMode::EnabledIfAvailable {
                            warn!("Could not find an appropriate validation layer. Check that the vulkan SDK has been installed or disable validation.");
                        } else {
                            error!("Could not find an appropriate validation layer. Check that the vulkan SDK has been installed or disable validation.");
                            return Err(vk::Result::ERROR_LAYER_NOT_PRESENT.into());
                        }
                    }
                };
            }
        };
        match windowing_mode {
            ExtensionMode::Disabled => {}
            ExtensionMode::EnabledIfAvailable | ExtensionMode::Enabled => {
                let window_extensions = Self::find_window_extensions(&extensions);
                if let Some(window_extensions) = window_extensions {
                    extension_names.extend_from_slice(&window_extensions);
                } else if validation_mode == ExtensionMode::EnabledIfAvailable {
                    warn!("Could not find the appropriate window extensions layers. Check that the appropriate drivers are installed");
                } else {
                    error!("Could not find the appropriate window extensions layers. Check that the appropriate drivers are installed");
                    return Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT.into());
                }
            }
        }
        Ok((layer_names, extension_names))
    }

    fn find_best_validation_layer(layers: &[ash::vk::LayerProperties]) -> Option<&'static CStr> {
        fn khronos_validation_layer_name() -> &'static CStr {
            CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
                .expect("Wrong extension string")
        }

        fn lunarg_validation_layer_name() -> &'static CStr {
            CStr::from_bytes_with_nul(b"VK_LAYER_LUNARG_standard_validation\0")
                .expect("Wrong extension string")
        }

        let khronos_validation_layer_name = khronos_validation_layer_name();
        let lunarg_validation_layer_name = lunarg_validation_layer_name();

        // Find the best validation layer that's available
        let mut best_available_layer = None;
        for layer in layers {
            let layer_name = unsafe { CStr::from_ptr(layer.layer_name.as_ptr()) };

            if layer_name == khronos_validation_layer_name {
                best_available_layer = Some(khronos_validation_layer_name);
                break;
            }

            if layer_name == lunarg_validation_layer_name {
                best_available_layer = Some(lunarg_validation_layer_name);
            }
        }

        best_available_layer
    }

    fn find_window_extensions(
        extensions: &[ash::vk::ExtensionProperties],
    ) -> Option<Vec<&'static CStr>> {
        #[cfg(target_os = "windows")]
        let platform_extensions = vec![khr::Surface::name(), khr::Win32Surface::name()];

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        let platform_extensions = vec![
            khr::Surface::name(),
            khr::WaylandSurface::name(),
            khr::XlibSurface::name(),
            khr::XcbSurface::name(),
        ];

        #[cfg(any(target_os = "android"))]
        let platform_extensions = vec![khr::Surface::name(), khr::AndroidSurface::name()];

        #[cfg(any(target_os = "macos"))]
        let platform_extensions = vec![khr::Surface::name(), ext::MetalSurface::name()];

        #[cfg(any(target_os = "ios"))]
        let platform_extensions = vec![khr::Surface::name(), ext::MetalSurface::name()];

        if check_extensions_availability(&platform_extensions, extensions) {
            Some(platform_extensions)
        } else {
            None
        }
    }
}

impl Drop for VkInstance {
    fn drop(&mut self) {
        trace!("destroying VkInstance");
        std::mem::drop(self.debug_reporter.take());

        unsafe {
            self.instance.destroy_instance(None);
        }

        trace!("destroyed VkInstance");
    }
}
