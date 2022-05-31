//! Additional devices
mod cas_device;

use std::sync::Arc;

use lgn_content_store::Provider;
use lgn_data_runtime::{manifest::ManifestId, AssetRegistryOptions};

/// Extend `AssetRegistryOptions` API, to add an offline CAS device
pub trait AddDeviceCASOffline {
    /// Adds a device that can read from a persistent content-store.
    /// Will skip over the resource meta-data.
    #[must_use]
    fn add_device_cas_offline(
        self,
        _provider: Arc<Provider>,
        _manifest_id: Arc<ManifestId>,
    ) -> Self;
}

impl AddDeviceCASOffline for AssetRegistryOptions {
    fn add_device_cas_offline(self, provider: Arc<Provider>, manifest_id: Arc<ManifestId>) -> Self {
        self.add_device(Box::new(cas_device::CasDevice::new(provider, manifest_id)))
    }
}
