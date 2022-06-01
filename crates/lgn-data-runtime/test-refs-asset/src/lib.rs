//! This module defines a test asset.
//!
//! It is used to test the data compilation process until we have a proper asset
//! available.

use async_trait::async_trait;
use lgn_data_model::implement_bincode_reader_writer;
use lgn_data_runtime::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
/// Asset temporarily used for testing.
///
/// To be removed once real asset types exist.
#[resource("refs_asset")]
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct RefsAsset {
    /// Test content.
    pub content: String,
    pub reference: Option<RefsAsssetReferenceType>,
}

lgn_data_model::implement_reference_type_def!(RefsAsssetReferenceType, RefsAsset);

impl RefsAsset {
    pub fn register_type(asset_registry_options: &mut AssetRegistryOptions) {
        ResourceType::register_name(
            <Self as ResourceDescriptor>::TYPE,
            <Self as ResourceDescriptor>::TYPENAME,
        );
        asset_registry_options.add_resource_installer(
            <Self as ResourceDescriptor>::TYPE,
            std::sync::Arc::new(RefsAssetLoader::default()),
        );
    }
    implement_bincode_reader_writer!();
}

#[derive(Default)]
struct RefsAssetLoader {}

#[async_trait]
impl crate::ResourceInstaller for RefsAssetLoader {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let mut refs_asset = RefsAsset::from_reader(reader).await?;

        // Manually active reference (reflection will do this on normal asset)
        if let Some(reference) = &mut refs_asset.reference {
            let child_id = reference.id();
            let child_handle = request.asset_registry.load_async_untyped(child_id).await?;
            reference.activate(child_handle);
        }

        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(refs_asset))?;
        Ok(handle)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ResourceId;
    use lgn_content_store::Provider;
    use lgn_data_runtime::{manifest::Manifest, AssetRegistry};
    use std::sync::Arc;

    async fn setup_singular_asset_test() -> (ResourceTypeAndId, Arc<AssetRegistry>) {
        let data_provider = std::sync::Arc::new(Provider::new_in_memory());
        let manifest = Manifest::default();

        let asset_id = {
            let type_id = ResourceTypeAndId {
                kind: RefsAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };

            let value = RefsAsset {
                content: "TestContent".into(),
                reference: None,
            };
            let content = value.to_bytes().unwrap();

            let checksum = data_provider.write(&content).await.unwrap();
            manifest.insert(type_id, checksum);
            type_id
        };

        let mut options = AssetRegistryOptions::new().add_device_cas(data_provider, manifest);
        RefsAsset::register_type(&mut options);
        let reg = options.create().await;

        (asset_id, reg)
    }

    #[tokio::test]
    async fn load_test_asset() {
        let (asset_id, reg) = setup_singular_asset_test().await;
        let internal_id;
        {
            let a = reg
                .load_async::<RefsAsset>(asset_id)
                .await
                .expect("failed to load BINARY_ASSETFILE");
            internal_id = a.id();
            a.get().unwrap();
            {
                let b = a.clone();
                assert_eq!(a, b);
                b.get().unwrap();
            }
        }
        reg.update();
        assert!(reg.lookup_untyped(&internal_id).is_none());
    }

    #[tokio::test]
    async fn load_error() {
        let (_, reg) = setup_singular_asset_test().await;
        let internal_id;
        {
            internal_id = ResourceTypeAndId {
                kind: RefsAsset::TYPE,
                id: ResourceId::new_explicit(7),
            };

            match reg.load_async::<RefsAsset>(internal_id).await {
                Ok(_handle) => panic!("Expected load error"),
                Err(_err) => (),
            }
        }
        reg.update();
        assert!(reg.lookup_untyped(&internal_id).is_none());
    }

    #[tokio::test]
    async fn reload_no_change() {
        let (asset_id, reg) = setup_singular_asset_test().await;

        let internal_id;
        {
            let a = reg.load_async::<RefsAsset>(asset_id).await.unwrap();
            internal_id = a.id();
            assert!(a.get().is_some());
            reg.reload(a.id()).await;
            assert!(a.get().is_some());
        }
        reg.update();
        assert!(reg.lookup_untyped(&internal_id).is_none());
    }

    async fn setup_dependency_test() -> (ResourceTypeAndId, ResourceTypeAndId, Arc<AssetRegistry>) {
        let data_provider = Arc::new(Provider::new_in_memory());
        let manifest = Manifest::default();

        let child_id = ResourceTypeAndId {
            kind: RefsAsset::TYPE,
            id: ResourceId::new_explicit(1),
        };
        let child_asset = RefsAsset {
            content: "ChildData".into(),
            reference: None,
        };
        manifest.insert(
            child_id,
            data_provider
                .write(&child_asset.to_bytes().unwrap())
                .await
                .unwrap(),
        );

        let parent_id = ResourceTypeAndId {
            kind: RefsAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };
        let parent_asset = RefsAsset {
            content: "ParentData".into(),
            reference: Some(RefsAsssetReferenceType::from(child_id)),
        };
        manifest.insert(
            parent_id,
            data_provider
                .write(&parent_asset.to_bytes().unwrap())
                .await
                .unwrap(),
        );

        let mut options = AssetRegistryOptions::new().add_device_cas(data_provider, manifest);
        RefsAsset::register_type(&mut options);
        let reg = options.create().await;

        (parent_id, child_id, reg)
    }

    #[tokio::test]
    async fn load_dependency() {
        let (parent_id, child_id, reg) = setup_dependency_test().await;

        let parent = reg.load_async::<RefsAsset>(parent_id).await.expect("");
        let child = reg
            .lookup::<RefsAsset>(&child_id)
            .expect("be loaded indirectly");
        std::mem::drop(parent);
        reg.update();

        assert!(reg.lookup_untyped(&parent_id).is_none());
        assert!(
            child.get().is_some(),
            "The dependency should be kept alive because of the handle"
        );
        std::mem::drop(child);
        reg.update();
        assert!(reg.lookup_untyped(&child_id).is_none());
    }
}
