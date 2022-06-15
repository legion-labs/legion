#![allow(clippy::type_complexity)]

use std::{collections::HashMap, sync::Arc};

use crate::{
    vfs, AssetRegistryError, AssetRegistryReader, HandleUntyped, LoadRequest, ResourceInstaller,
    ResourceType, ResourceTypeAndId,
};

pub(crate) struct AssetLoaderIO {
    devices: Vec<Box<(dyn vfs::Device + Send)>>,
    installers: HashMap<ResourceType, Arc<dyn ResourceInstaller>>,
}

impl AssetLoaderIO {
    pub(crate) fn new(
        devices: Vec<Box<(dyn vfs::Device + Send)>>,
        installers: HashMap<ResourceType, Arc<dyn ResourceInstaller>>,
    ) -> Self {
        Self {
            devices,
            installers,
        }
    }

    pub(crate) async fn load_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        mut reader: AssetRegistryReader,
        request: &mut LoadRequest,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        lgn_tracing::debug!("Loading Request {:?}", resource_id);
        let installer = self.installers.get(&resource_id.kind).ok_or(
            AssetRegistryError::ResourceInstallerNotFound(resource_id.kind),
        )?;

        let new_resource = installer
            .install_from_stream(resource_id, request, &mut reader)
            .await?;

        let handle = request
            .asset_registry
            .set_resource(resource_id, new_resource)?;

        Ok(handle)
    }

    /// Load a resource using async framework
    pub(crate) async fn load_from_device(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        for device in &self.devices {
            if let Some(reader) = device.get_reader(resource_id).await {
                return self.load_from_stream(resource_id, reader, request).await;
            }
        }
        Err(AssetRegistryError::ResourceNotFound(resource_id))
    }
}

/*
#[cfg(test)]
mod tests {
    use generic_data::TestAsset;
    use std::{sync::Arc, time::Duration};

    use lgn_content_store::{
        indexing::{ResourceIndex, ResourceWriter, SharedTreeIdentifier},
        Provider,
    };

    use super::AssetLoaderIO;
    use crate::{
        new_resource_type_and_id_indexer, vfs, Handle, ResourceDescriptor, ResourceId,
        ResourceTypeAndId,
    };

    async fn setup_test() -> (ResourceTypeAndId, AssetLoaderIO) {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest =
            ResourceIndex::new_exclusive(new_resource_type_and_id_indexer(), &data_provider).await;

        let asset_id = {
            let type_id = ResourceTypeAndId {
                kind: TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let provider_id = data_provider
                .write_resource_from_bytes(&tests::BINARY_ASSETFILE)
                .await
                .unwrap();

            manifest
                .add_resource(&data_provider, &type_id.into(), provider_id)
                .await
                .unwrap();

            type_id
        };

        let (loader, mut io) = create_loader(vec![Box::new(vfs::CasDevice::new(
            Arc::clone(&data_provider),
            SharedTreeIdentifier::new(manifest.id()),
        ))]);
        io.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        (asset_id, loader, io)
    }

    #[tokio::test]
    async fn typed_ref() {
        let (asset_id, loader, mut io) = setup_test().await;

        {
            let untyped = loader.load(asset_id);

            assert_eq!(untyped.id(), asset_id);

            let typed: Handle<test_asset::TestAsset> = untyped.into();

            io.wait(Duration::from_millis(500)).await;
            assert!(loader.handles.pin().get(&typed.id()).is_some());

            match loader.try_result() {
                Some(LoaderResult::Loaded(handle, _, _)) => {
                    assert!(handle.id() == typed.id());
                }
                _ => panic!(),
            }
        }

        loader.collect_dropped_handles();

        assert!(!loader.handles.pin().contains_key(&asset_id));

        let typed: Handle<test_asset::TestAsset> = loader.load(asset_id).into();
        io.wait(Duration::from_millis(500)).await;

        assert!(loader.handles.pin().get(&typed.id()).is_some());
    }

    #[tokio::test]
    async fn call_load_twice() {
        let (asset_id, loader, _io) = setup_test().await;

        let a = loader.load(asset_id);
        {
            let b = a.clone();
            assert_eq!(a.id(), b.id());
            {
                let c = loader.load(asset_id);
                assert_eq!(a.id(), c.id());
            }
        }
    }

    #[tokio::test]
    async fn load_no_dependencies() {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest =
            ResourceIndex::new_exclusive(new_resource_type_and_id_indexer(), &data_provider).await;

        let asset_id = {
            let type_id = ResourceTypeAndId {
                kind: TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let provider_id = data_provider
                .write_resource_from_bytes(&tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest
                .add_resource(&data_provider, &type_id.into(), provider_id)
                .await
                .unwrap();
            type_id
        };

        let (unload_tx, _unload_rx) = crossbeam_channel::unbounded::<_>();

        let handles = HandleMap::new(unload_tx);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                data_provider,
                SharedTreeIdentifier::new(manifest.id()),
            ))],
            request_tx.clone(),
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_handle, load_id))
            .expect("to send request");

        assert!(!loader.loaded_resources.contains(&asset_id));

        let mut result = None;
        loader.wait(Duration::from_millis(1)).await;
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_, _, _)));
        assert!(loader.loaded_resources.contains(&asset_id));

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&asset_id));
    }

    #[tokio::test]
    async fn load_failed_dependency() {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest =
            ResourceIndex::new_exclusive(new_resource_type_and_id_indexer(), &data_provider).await;

        let parent_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };

        let asset_id = {
            let provider_id = data_provider
                .write_resource_from_bytes(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            manifest
                .add_resource(&data_provider, &parent_id.into(), provider_id)
                .await
                .unwrap();
            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                data_provider,
                SharedTreeIdentifier::new(manifest.id()),
            ))],
            request_tx.clone(),
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_handle, load_id))
            .expect("valid tx");

        assert!(!loader.loaded_resources.contains(&asset_id));

        let mut result = None;
        loader.wait(Duration::from_millis(1)).await;
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::LoadError(_, _, _)));
    }

    #[tokio::test]
    async fn load_with_dependency() {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest =
            ResourceIndex::new_exclusive(new_resource_type_and_id_indexer(), &data_provider).await;

        let parent_content = "parent";

        let parent_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };
        let child_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(1),
        };

        let asset_id = {
            manifest
                .add_resource(
                    &data_provider,
                    &child_id.into(),
                    data_provider
                        .write_resource_from_bytes(&test_asset::tests::BINARY_ASSETFILE)
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            let provider_id = data_provider
                .write_resource_from_bytes(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            manifest
                .add_resource(&data_provider, &parent_id.into(), provider_id)
                .await
                .unwrap();

            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                data_provider,
                SharedTreeIdentifier::new(manifest.id()),
            ))],
            request_tx.clone(),
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        let load_id = Some(0);
        request_tx
            .send(LoaderRequest::Load(asset_handle.clone(), load_id))
            .expect("to send request");

        assert!(!loader.loaded_resources.contains(&parent_id));

        let mut result = None;
        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

        if let Ok(res) = result_rx.try_recv() {
            // child load result comes first with no load_id..
            assert!(matches!(res, LoaderResult::Loaded(_, _, None)));
        }

        if let Ok(res) = result_rx.try_recv() {
            // ..followed by parent load result with load_id
            assert!(matches!(res, LoaderResult::Loaded(_, _, Some(_))));
            result = Some(res);
        }

        assert!(result.is_some());
        let result = result.unwrap();
        assert!(matches!(result, LoaderResult::Loaded(_, _, _)));
        assert!(loader.loaded_resources.contains(&parent_id));
        assert!(loader.loaded_resources.contains(&child_id));

        if let LoaderResult::Loaded(handle, asset, returned_load_id) = result {
            let asset = asset.downcast_ref::<test_asset::TestAsset>().unwrap();
            assert_eq!(asset.content, parent_content);
            assert_eq!(asset_handle, handle);
            assert_eq!(returned_load_id, load_id);
        }

        // unload and validate references.

        request_tx
            .send(LoaderRequest::Unload(parent_id))
            .expect("to send request");

        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&parent_id));

        /*
            assert_eq!(result.assets.len(), 1);
            assert_eq!(result._load_dependencies.len(), 1);

            let (asset_id, asset) = &result.assets[0];

            let asset = asset.downcast_ref::<TestAsset>().unwrap();
            assert_eq!(asset.content, expected_content);
            assert_eq!(asset_id, &id);
        */
    }

    #[tokio::test]
    async fn reload_no_dependencies() {
        let data_provider = Arc::new(Provider::new_in_memory());
        let mut manifest =
            ResourceIndex::new_exclusive(new_resource_type_and_id_indexer(), &data_provider).await;

        let asset_id = {
            let type_id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let provider_id = data_provider
                .write_resource_from_bytes(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest
                .add_resource(&data_provider, &type_id.into(), provider_id)
                .await
                .unwrap();
            type_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Arc::clone(&data_provider),
                SharedTreeIdentifier::new(manifest.id()),
            ))],
            request_tx.clone(),
        );
        loader.register_loader(
            test_asset::TestAsset::TYPE,
            Box::new(test_asset::TestAssetLoader {}),
        );

        request_tx
            .send(LoaderRequest::Load(asset_handle.clone(), None))
            .expect("to send request");

        assert!(!loader.loaded_resources.contains(&asset_id));

        let mut result = None;
        loader.wait(Duration::from_millis(1)).await;
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }

        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Loaded(_, _, _)));
        assert!(loader.loaded_resources.contains(&asset_id));

        // reload
        request_tx
            .send(LoaderRequest::Reload(asset_handle))
            .unwrap();

        let mut result = None;
        loader.wait(Duration::from_millis(10)).await;
        if let Ok(res) = result_rx.try_recv() {
            result = Some(res);
        }
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), LoaderResult::Reloaded(_, _)));

        // unload and validate references.
        request_tx
            .send(LoaderRequest::Unload(asset_id))
            .expect("valid tx");

        while loader.wait(Duration::from_millis(1)).await.unwrap() > 0 {}

        assert!(!loader.loaded_resources.contains(&asset_id));
    }
}
*/
