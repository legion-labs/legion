use async_trait::async_trait;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryReader, HandleUntyped, LoadRequest, Resource,
    ResourceInstaller, ResourceTypeAndId,
};
use tokio::io::AsyncReadExt;

#[resource("integer_asset")]
#[derive(Clone)]
pub struct IntegerAsset {
    pub magic_value: i32,
}

impl IntegerAsset {
    /// # Errors
    /// return a `AssetRegistryError` if it failed to create a `IntegerAsset` from an async reader
    pub async fn from_reader(reader: &mut AssetRegistryReader) -> Result<Self, AssetRegistryError> {
        let mut buf = 0i32.to_ne_bytes();
        reader.read_exact(&mut buf).await?;
        let magic_value = i32::from_ne_bytes(buf);
        Ok(Self { magic_value })
    }
}
#[derive(Default)]
struct IntegerAssetLoader {}

#[async_trait]
impl ResourceInstaller for IntegerAssetLoader {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let handle = request.asset_registry.set_resource(
            resource_id,
            Box::new(IntegerAsset::from_reader(reader).await?),
        )?;
        Ok(handle)
    }
}
