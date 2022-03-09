use std::{any::Any, io, path::Path};

use crate::{
    helpers::{read_u32, write_u32},
    offline::{Material, Mesh, Model},
    offline_texture::{Texture, TextureType},
    Color,
};
use gltf::{
    image::Format,
    mesh::util::{ReadIndices, ReadTexCoords},
    Document,
};
use lgn_math::{Vec2, Vec3};

use lgn_data_offline::{
    resource::{OfflineResource, ResourceProcessor, ResourceProcessorError},
    ResourcePathId,
};
use lgn_data_runtime::{
    resource, Asset, AssetLoader, AssetLoaderError, Resource, ResourceTypeAndId,
};

use crate::helpers::{read_usize, read_usize_and_buffer, write_usize, write_usize_and_buffer};

#[resource("gltf")]
#[derive(Default)]
pub struct GltfFile {
    pub document: Option<Document>,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

impl GltfFile {
    pub fn from_path(path: &Path) -> Self {
        let (document, buffers, images) = gltf::import(path).unwrap();
        Self {
            document: Some(document),
            buffers,
            images,
        }
    }

    pub fn gather_models(&self, resource_id: ResourceTypeAndId) -> Vec<(Model, String)> {
        let mut models = Vec::new();
        for model in self.document.as_ref().unwrap().meshes() {
            let mut meshes = Vec::new();
            for primitive in model.primitives() {
                let mut positions: Vec<Vec3> = Vec::new();
                let mut normals: Vec<Vec3> = Vec::new();
                let mut tangents: Vec<Vec3> = Vec::new();
                let mut tex_coords: Vec<Vec2> = Vec::new();
                let mut indices: Vec<u16> = Vec::new();

                let reader = primitive.reader(|buffer| Some(&self.buffers[buffer.index()]));
                if let Some(iter) = reader.read_positions() {
                    for vertex_position in iter {
                        positions.push(vertex_position.into());
                    }
                }
                if let Some(iter) = reader.read_normals() {
                    for normal in iter {
                        normals.push(normal.into());
                    }
                }
                if let Some(iter) = reader.read_tangents() {
                    for tangent in iter {
                        tangents.push(Vec3::new(tangent[0], tangent[1], tangent[2]));
                    }
                }
                if let Some(tex_coords_option) = reader.read_tex_coords(0) {
                    match tex_coords_option {
                        ReadTexCoords::F32(iter) => {
                            for tex_coord in iter {
                                tex_coords.push(Vec2::new(tex_coord[0], tex_coord[1]));
                            }
                        }
                        _ => unreachable!("Integer UVs are not supported"),
                    }
                }
                if let Some(indices_option) = reader.read_indices() {
                    match indices_option {
                        ReadIndices::U8(iter) => {
                            for idx in iter {
                                indices.push(u16::from(idx));
                            }
                        }
                        ReadIndices::U16(iter) => {
                            for idx in iter {
                                indices.push(idx);
                            }
                        }
                        ReadIndices::U32(iter) => {
                            for idx in iter {
                                // TODO - will panic if does not fit in 16bits
                                indices.push(idx as u16);
                            }
                        }
                    }
                }

                let mut indices = Some(indices);
                if tangents.is_empty() && !normals.is_empty() {
                    tangents = lgn_math::calculate_tangents(&positions, &tex_coords, &indices);
                }

                let mut material = primitive.material().name().map(|material_name| {
                    ResourcePathId::from(resource_id)
                        .push_named(crate::offline::Material::TYPE, material_name)
                        .push(crate::runtime::Material::TYPE)
                });
                if material.is_none() {
                    material = primitive.material().index().map(|idx| {
                        ResourcePathId::from(resource_id)
                            .push_named(
                                crate::offline::Material::TYPE,
                                self.document
                                    .as_ref()
                                    .unwrap()
                                    .materials()
                                    .nth(idx)
                                    .unwrap()
                                    .name()
                                    .unwrap(),
                            )
                            .push(crate::runtime::Material::TYPE)
                    });
                }
                meshes.push(Mesh {
                    positions,
                    normals,
                    tangents,
                    tex_coords,
                    indices: indices.take().unwrap(),
                    colors: Vec::new(),
                    material,
                });
            }
            models.push((Model { meshes }, String::from(model.name().unwrap())));
        }
        models
    }

    pub fn gather_materials(&self, resource_id: ResourceTypeAndId) -> Vec<(Material, String)> {
        let mut materials = Vec::new();
        for material in self.document.as_ref().unwrap().materials() {
            let base_albedo = material.pbr_metallic_roughness().base_color_factor();
            let base_albedo = Color::from((
                (base_albedo[0] * 255.0) as u8,
                (base_albedo[1] * 255.0) as u8,
                (base_albedo[2] * 255.0) as u8,
                (base_albedo[3] * 255.0) as u8,
            ));
            let albedo = material
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|info| {
                    info.texture().name().map(|texture_name| {
                        ResourcePathId::from(resource_id)
                            .push_named(crate::offline_texture::Texture::TYPE, texture_name)
                            .push_named(crate::runtime_texture::Texture::TYPE, "Albedo")
                    })
                });
            let albedo = if let Some(albedo) = albedo {
                albedo
            } else {
                None
            };

            let normal = material.normal_texture().map(|info| {
                info.texture().name().map(|texture_name| {
                    ResourcePathId::from(resource_id)
                        .push_named(crate::offline_texture::Texture::TYPE, texture_name)
                        .push_named(crate::runtime_texture::Texture::TYPE, "Normal")
                })
            });
            let normal = if let Some(normal) = normal {
                normal
            } else {
                None
            };
            let base_roughness = material.pbr_metallic_roughness().roughness_factor();
            let base_metalness = material.pbr_metallic_roughness().metallic_factor();
            let roughness = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
                .map(|info| {
                    info.texture().name().map(|texture_name| {
                        ResourcePathId::from(resource_id)
                            .push_named(
                                crate::offline_texture::Texture::TYPE,
                                format!("{}_Roughness", texture_name).as_str(),
                            )
                            .push_named(crate::runtime_texture::Texture::TYPE, "Roughness")
                    })
                });
            let roughness = if let Some(roughness) = roughness {
                roughness
            } else {
                None
            };
            let metalness = material
                .pbr_metallic_roughness()
                .metallic_roughness_texture()
                .map(|info| {
                    info.texture().name().map(|texture_name| {
                        ResourcePathId::from(resource_id)
                            .push_named(
                                crate::offline_texture::Texture::TYPE,
                                format!("{}_Metalness", texture_name).as_str(),
                            )
                            .push_named(crate::runtime_texture::Texture::TYPE, "Metalness")
                    })
                });
            let metalness = if let Some(metalness) = metalness {
                metalness
            } else {
                None
            };
            materials.push((
                Material {
                    albedo,
                    normal,
                    roughness,
                    metalness,
                    base_albedo,
                    base_metalness,
                    base_roughness,
                    ..Material::default()
                },
                String::from(material.name().unwrap()),
            ));
        }
        materials
    }

    pub fn gather_textures(&self) -> Vec<(Texture, String)> {
        let mut textures = Vec::new();
        for texture in self.document.as_ref().unwrap().textures() {
            let name = texture.name().unwrap();
            let image = &self.images[texture.source().index()];
            if name.contains("OcclusionRoughMetal") {
                let mut roughness = Vec::new();
                let mut metalness = Vec::new();
                for i in 0..(image.width * image.height) as usize {
                    roughness.push(image.pixels[i * 3 + 1]);
                    metalness.push(image.pixels[i * 3 + 2]);
                }
                textures.push((
                    Texture {
                        kind: TextureType::_2D,
                        width: image.width,
                        height: image.height,
                        rgba: match image.format {
                            Format::R8G8B8A8 => roughness,
                            _ => unreachable!(),
                        },
                    },
                    format!("{}_Roughness", name),
                ));
                textures.push((
                    Texture {
                        kind: TextureType::_2D,
                        width: image.width,
                        height: image.height,
                        rgba: match image.format {
                            Format::R8G8B8A8 => metalness,
                            _ => unreachable!(),
                        },
                    },
                    format!("{}_Metalness", name),
                ));
            } else {
                textures.push((
                    Texture {
                        kind: TextureType::_2D,
                        width: image.width,
                        height: image.height,
                        rgba: match image.format {
                            //Format::R8 => image.pixels.clone().iter().flat_map(|v| vec![*v, 0, 0, 0]).collect(),
                            Format::R8G8B8A8 => {
                                let mut rgba = Vec::new();
                                for i in 0..(image.width * image.height) as usize {
                                    rgba.push(image.pixels[i * 3]);
                                    rgba.push(image.pixels[i * 3 + 1]);
                                    rgba.push(image.pixels[i * 3 + 2]);
                                    rgba.push(255);
                                }
                                rgba
                            }
                            _ => unreachable!(),
                        },
                    },
                    String::from(name),
                ));
            }
        }
        textures
    }
}

impl Asset for GltfFile {
    type Loader = GltfFileProcessor;
}

impl OfflineResource for GltfFile {
    type Processor = GltfFileProcessor;
}

#[derive(Default)]
pub struct GltfFileProcessor {}

impl AssetLoader for GltfFileProcessor {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&mut buffer)).map_err(|err| {
            AssetLoaderError::ErrorLoading("GLTF", format!("Couldn't read zip archive: {}", err))
        })?;
        let mut zip_file = zip.by_index(0).map_err(|err| {
            AssetLoaderError::ErrorLoading("GLTF", format!("No file in zip archive: {}", err))
        })?;
        let document_bytes = read_usize_and_buffer(&mut zip_file)?;
        let buffers_length = read_usize(&mut zip_file)?;
        let mut buffers = Vec::with_capacity(buffers_length);
        for _i in 0..buffers_length {
            let buffer = read_usize_and_buffer(&mut zip_file)?;
            buffers.push(gltf::buffer::Data(buffer));
        }
        let images_length = read_usize(&mut zip_file)?;
        let mut images = Vec::with_capacity(images_length);
        for _i in 0..images_length {
            let mut image = gltf::image::Data {
                format: Format::R8G8B8A8,
                width: 0,
                height: 0,
                pixels: Vec::new(),
            };
            image.width = read_u32(&mut zip_file)?;
            image.height = read_u32(&mut zip_file)?;
            //TODO: read format
            image.pixels = read_usize_and_buffer(&mut zip_file)?;
            images.push(image);
        }

        let document = Document::from_json(
            gltf::json::deserialize::from_slice::<'_, gltf::json::Root>(&document_bytes).map_err(
                |err| {
                    AssetLoaderError::ErrorLoading(
                        "GLTF",
                        format!("Couldn't deserialize json: {}", err),
                    )
                },
            )?,
        )
        .map_err(|err| {
            AssetLoaderError::ErrorLoading(
                "GLTF",
                format!("Couldn't create document out of json: {}", err),
            )
        })?;
        Ok(Box::new(GltfFile {
            document: Some(document),
            buffers,
            images,
        }))
    }

    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for GltfFileProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(GltfFile::default())
    }

    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        Vec::new()
    }

    fn write_resource(
        &self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> Result<usize, ResourceProcessorError> {
        let gltf = resource.downcast_ref::<GltfFile>().unwrap();
        if gltf.document.is_none() {
            return Ok(0);
        }
        let mut buffer = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("GltfFile.zip", options).map_err(|err| {
            ResourceProcessorError::ResourceSerializationFailed(
                "GLTF",
                format!("Couldn't start zip: {}", err),
            )
        })?;
        let document_bytes =
            gltf::json::serialize::to_vec(&gltf.document.clone().unwrap().into_json()).unwrap();
        let mut written = write_usize_and_buffer(&mut zip, &document_bytes)?;
        let buffer_length = gltf.buffers.len();
        written += write_usize(&mut zip, buffer_length)?;
        for buffer in &gltf.buffers {
            written += write_usize_and_buffer(&mut zip, &buffer.0)?;
        }

        let image_length = gltf.images.len();
        written += write_usize(&mut zip, image_length)?;
        for image in &gltf.images {
            written += write_u32(&mut zip, &image.width)?;
            written += write_u32(&mut zip, &image.height)?;
            //TODO: written += format
            written += write_usize_and_buffer(&mut zip, &image.pixels)?;
        }
        zip.finish().map_err(|err| {
            ResourceProcessorError::ResourceSerializationFailed(
                "GLTF",
                format!("Couldn't finish zip: {}", err),
            )
        })?;
        drop(zip);
        written = writer.write(&buffer)?;
        Ok(written)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
