#![allow(clippy::type_complexity)]

use std::{collections::HashMap, sync::Arc};

use lgn_content_store::Identifier;

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

        let new_handle = installer
            .install_from_stream(resource_id, request, &mut reader)
            .await?;

        Ok(new_handle)
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

    async fn _process_load_manifest(&mut self, manifest_id: &Identifier) {
        for device in &mut self.devices {
            device.reload_manifest(manifest_id).await;
        }
    }
}

/*
#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use lgn_content_store::{ContentProvider, ContentWriterExt, MemoryProvider};

    use super::{create_loader, AssetLoaderIO, AssetLoaderStub};
    use crate::{
        asset_loader::{HandleMap, LoaderRequest, LoaderResult},
        manifest::Manifest,
        test_asset, vfs, Handle, ResourceDescriptor, ResourceId, ResourceTypeAndId,
    };

    async fn setup_test() -> (ResourceTypeAndId, AssetLoaderStub, AssetLoaderIO) {
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(id, checksum);
            id
        };

        let (loader, mut io) = create_loader(vec![Box::new(vfs::CasDevice::new(
            Some(manifest),
            Arc::clone(&data_content_provider),
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
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(id, checksum);
            id
        };

        let (unload_tx, _unload_rx) = crossbeam_channel::unbounded::<_>();

        let handles = HandleMap::new(unload_tx);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                data_content_provider,
            ))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
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
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let parent_id = ResourceTypeAndId {
            kind: test_asset::TestAsset::TYPE,
            id: ResourceId::new_explicit(2),
        };

        let asset_id = {
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(parent_id, checksum);
            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                data_content_provider,
            ))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
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
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

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
            manifest.insert(
                child_id,
                data_content_provider
                    .write_content(&test_asset::tests::BINARY_ASSETFILE)
                    .await
                    .unwrap(),
            );
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_PARENT_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(parent_id, checksum);

            parent_id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                data_content_provider,
            ))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
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
        let data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>> =
            Arc::new(Box::new(MemoryProvider::new()));
        let manifest = Manifest::default();

        let asset_id = {
            let id = ResourceTypeAndId {
                kind: test_asset::TestAsset::TYPE,
                id: ResourceId::new_explicit(1),
            };
            let checksum = data_content_provider
                .write_content(&test_asset::tests::BINARY_ASSETFILE)
                .await
                .unwrap();
            manifest.insert(id, checksum);
            id
        };

        let handles = HandleMap::new(crossbeam_channel::unbounded::<_>().0);
        let asset_handle = handles.create_handle(asset_id);

        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel::<LoaderRequest>();
        let (result_tx, result_rx) = crossbeam_channel::unbounded::<LoaderResult>();
        let mut loader = AssetLoaderIO::new(
            vec![Box::new(vfs::CasDevice::new(
                Some(manifest),
                Arc::clone(&data_content_provider),
            ))],
            request_tx.clone(),
            request_rx,
            result_tx,
            handles,
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
