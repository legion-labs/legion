//! A module providing runtime texture related functionality.

use async_trait::async_trait;
use lgn_data_model::implement_reference_type_def;
use lgn_data_runtime::{
    resource, AssetRegistryError, AssetRegistryReader, HandleUntyped, LoadRequest, Resource,
    ResourceDescriptor, ResourceInstaller, ResourceTypeAndId,
};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use crate::{encode_mip_chain_from_offline_texture, TextureFormat};

/// Runtime texture.
#[resource("runtime_texture")]
#[derive(Serialize, Deserialize, Clone)]
pub struct Texture {
    /// Texture width.
    pub width: u32,
    /// Texture height.
    pub height: u32,
    /// Desired HW texture format
    pub format: TextureFormat,
    /// Color encoding
    pub srgb: bool,
    /// Mip chain pixel data of the image in hardware encoded form
    pub texture_data: Vec<serde_bytes::ByteBuf>,
}

impl Texture {
    pub fn compile_from_offline(
        width: u32,
        height: u32,
        format: TextureFormat,
        srgb: bool,
        alpha_blended: bool,
        rgba: &[u8],
        writer: &mut dyn std::io::Write,
    ) {
        let texture = Self {
            width,
            height,
            format,
            srgb,
            texture_data: encode_mip_chain_from_offline_texture(
                width,
                height,
                format,
                alpha_blended,
                rgba,
            )
            .into_iter()
            .map(serde_bytes::ByteBuf::from)
            .collect::<Vec<_>>(),
        };
        bincode::serialize_into(writer, &texture).unwrap();
    }

    /// # Errors
    /// return a `AssetRegistryError` if it failed to create a `Texture` from an async reader
    pub async fn from_reader(reader: &mut AssetRegistryReader) -> Result<Self, AssetRegistryError> {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;
        let texture: Self = bincode::deserialize_from(&mut buffer.as_slice()).map_err(|err| {
            AssetRegistryError::ResourceSerializationFailed(Self::TYPENAME, err.to_string())
        })?;
        Ok(texture)
    }
}

implement_reference_type_def!(TextureReferenceType, Texture);

/// Loader of [`Texture`].
#[derive(Default)]
struct TextureLoader {}

#[async_trait]
impl ResourceInstaller for TextureLoader {
    async fn install_from_stream(
        &self,
        resource_id: ResourceTypeAndId,
        request: &mut LoadRequest,
        reader: &mut AssetRegistryReader,
    ) -> Result<HandleUntyped, AssetRegistryError> {
        let handle = request
            .asset_registry
            .set_resource(resource_id, Box::new(Texture::from_reader(reader).await?))?;

        Ok(handle)
    }
}
