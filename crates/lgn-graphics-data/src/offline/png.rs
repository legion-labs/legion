use lgn_data_offline::resource::{OfflineResource, ResourceProcessor, ResourceProcessorError};
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};
use serde::{Deserialize, Serialize};
use std::{any::Any, fs::File, io, path::Path};

use crate::{
    bcn_encoder::{ColorChannels, CompressionQuality, TextureFormat},
    offline_texture::{Texture, TextureType},
};

/// PNG Document file.
#[resource("png")]
#[derive(Serialize, Deserialize)]
pub struct PngFile {
    width: u32,
    height: u32,
    color_channels: ColorChannels,
    data: Vec<u8>,
}

impl Default for PngFile {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            color_channels: ColorChannels::Rgba,
            data: vec![],
        }
    }
}

impl Asset for PngFile {
    type Loader = PngFileProcessor;
}

impl OfflineResource for PngFile {
    type Processor = PngFileProcessor;
}

impl PngFile {
    pub fn from_file_path(file: &Path) -> Option<Self> {
        let decoder = png::Decoder::new(File::open(file).ok()?);
        let mut reader = decoder.read_info().ok()?;
        let mut img_data = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut img_data).ok()?;

        if info.color_type != png::ColorType::Indexed {
            Some(Self {
                width: info.width,
                height: info.height,
                color_channels: match info.color_type {
                    png::ColorType::Grayscale => ColorChannels::R,
                    png::ColorType::Rgb | png::ColorType::Indexed => ColorChannels::Rgb,
                    png::ColorType::GrayscaleAlpha => ColorChannels::Ra,
                    png::ColorType::Rgba => ColorChannels::Rgba,
                },
                data: img_data,
            })
        } else {
            None
        }
    }

    /// # Errors
    ///
    /// Will return `Err` if there is no PNG data loaded or the data uses a palette index.
    pub fn as_texture(&self) -> Texture {
        Texture {
            kind: TextureType::_2D,
            format: TextureFormat::BC1,
            quality: CompressionQuality::Fast,
            width: self.width,
            height: self.height,
            color_channels: self.color_channels,
            rgba: self.data.clone(),
        }
    }
}

/// A processor of PNG files.
#[derive(Default)]
pub struct PngFileProcessor {}

impl AssetLoader for PngFileProcessor {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        if let Ok(file) = bincode::deserialize_from::<&mut dyn io::Read, PngFile>(reader) {
            Ok(Box::new(file))
        } else {
            Ok(Box::new(PngFile::default()))
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for PngFileProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(PngFile::default())
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        vec![]
    }

    fn write_resource(
        &self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let png = resource.downcast_ref::<PngFile>().unwrap();
        bincode::serialize_into(writer, &png).unwrap();
        Ok(1)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
