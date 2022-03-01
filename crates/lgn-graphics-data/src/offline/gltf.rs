use std::{any::Any, io, path::Path};

use crate::offline::{Mesh, Model};
use gltf::{
    mesh::util::{ReadIndices, ReadTexCoords},
    Document,
};
use lgn_math::{Vec2, Vec3, Vec4};

use lgn_data_offline::resource::{OfflineResource, ResourceProcessor, ResourceProcessorError};
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};

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

    pub fn gather_models(&self) -> Vec<(Model, String)> {
        let mut models = Vec::new();
        for model in self.document.as_ref().unwrap().meshes() {
            let mut meshes = Vec::new();
            for primitive in model.primitives() {
                let mut positions: Vec<Vec3> = Vec::new();
                let mut normals: Vec<Vec3> = Vec::new();
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

                let positions = positions
                    .into_iter()
                    .map(|v: Vec3| Vec4::new(v.x, v.y, v.z, 1.0))
                    .collect::<Vec<Vec4>>();
                let normals = normals
                    .into_iter()
                    .map(|v: Vec3| Vec4::new(v.x, v.y, v.z, 0.0))
                    .collect();
                let mut indices = Some(indices);
                let tangents = lgn_math::calculate_tangents(&positions, &tex_coords, &indices);

                meshes.push(Mesh {
                    positions,
                    normals,
                    tangents,
                    tex_coords,
                    indices: indices.take().unwrap(),
                    colors: Vec::new(),
                    material: None,
                });
            }
            models.push((Model { meshes }, String::from(model.name().unwrap())));
        }
        models
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
        let result = read_usize_and_buffer(reader);
        if result.is_err() {
            return Ok(Box::new(GltfFile::default()));
        }
        let document_bytes = result.unwrap();
        let buffers_length = read_usize(reader)?;
        let mut buffers = Vec::new();
        for _i in 0..buffers_length {
            let buffer = read_usize_and_buffer(reader)?;
            buffers.push(gltf::buffer::Data(buffer));
        }

        let document = Document::from_json(
            gltf::json::deserialize::from_slice::<'_, gltf::json::Root>(&document_bytes).unwrap(),
        )
        .unwrap();
        Ok(Box::new(GltfFile {
            document: Some(document),
            buffers,
            images: Vec::new(),
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
        vec![]
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
        let document_bytes =
            gltf::json::serialize::to_vec(&gltf.document.clone().unwrap().into_json()).unwrap();
        let mut written = write_usize_and_buffer(writer, &document_bytes)?;
        let buffer_length = gltf.buffers.len();
        written += write_usize(writer, buffer_length)?;
        for buffer in &gltf.buffers {
            written += write_usize_and_buffer(writer, &buffer.0)?;
        }
        // TODO: image loading support
        //let image_length = gltf.images.len();
        //written += write_usize(writer, image_length)?;
        //for image in &gltf.images {
        //    written += write_usize_and_buffer(writer, &image.0)?;
        //}
        Ok(written)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
