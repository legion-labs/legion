//! Additional devices
mod source_cas_device;

use std::sync::Arc;

use lgn_content_store::{indexing::SharedTreeIdentifier, Provider};
use lgn_data_runtime::AssetRegistryOptions;

/// Extend `AssetRegistryOptions` API, to add an offline CAS device
pub trait AddDeviceSourceCas {
    /// Adds a device that can read from a persistent content-store.
    /// Will skip over the resource meta-data.
    #[must_use]
    fn add_device_source_cas(
        self,
        _provider: Arc<Provider>,
        _manifest_id: SharedTreeIdentifier,
    ) -> Self;
}

impl AddDeviceSourceCas for AssetRegistryOptions {
    fn add_device_source_cas(
        self,
        provider: Arc<Provider>,
        manifest_id: SharedTreeIdentifier,
    ) -> Self {
        self.add_device(Box::new(source_cas_device::SourceCasDevice::new(
            provider,
            manifest_id,
        )))
    }
}
