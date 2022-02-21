use std::{any::Any, io, path::Path};

use crate::runtime::{Mesh, SubMesh};
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

    pub fn new_mesh(&self) -> Vec<(Mesh, String)> {
        let mut meshes = Vec::new();
        for mesh in self.document.as_ref().unwrap().meshes() {
            let mut submeshes = Vec::new();
            for primitive in mesh.primitives() {
                let mut positions: Vec<Vec3> = Vec::new();
                let mut normals: Vec<Vec3> = Vec::new();
                let mut tex_coords: Vec<Vec2> = Vec::new();
                let mut indices: Vec<u32> = Vec::new();

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
                                indices.push(u32::from(idx));
                            }
                        }
                        ReadIndices::U16(iter) => {
                            for idx in iter {
                                indices.push(u32::from(idx));
                            }
                        }
                        ReadIndices::U32(iter) => {
                            for idx in iter {
                                indices.push(idx);
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
                let indices = Some(indices);
                let tangents = calculate_tangents(&positions, &tex_coords, &indices);
                submeshes.push(SubMesh {
                    positions: Some(positions),
                    normals: Some(normals),
                    tangents: Some(tangents),
                    tex_coords: Some(tex_coords),
                    indices,
                    colors: None,
                    material: None,
                });
            }
            meshes.push((Mesh { submeshes }, String::from(mesh.name().unwrap())));
        }
        meshes
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

#[allow(unsafe_code, clippy::uninit_vec)]
fn calculate_tangents(
    positions: &[Vec4],
    tex_coords: &[Vec2],
    indices: &Option<Vec<u32>>,
) -> Vec<Vec4> {
    let length = positions.len();
    let mut tangents = Vec::with_capacity(length);
    //let mut bitangents = Vec::with_capacity(length);
    unsafe {
        tangents.set_len(length);
        //bitangents.set_len(length);
    }

    let num_triangles = if let Some(indices) = &indices {
        indices.len() / 3
    } else {
        length / 3
    };

    for i in 0..num_triangles {
        let idx0 = if let Some(indices) = &indices {
            indices[i * 3] as usize
        } else {
            i * 3
        };
        let idx1 = if let Some(indices) = &indices {
            indices[i * 3 + 1] as usize
        } else {
            i * 3 + 1
        };
        let idx2 = if let Some(indices) = &indices {
            indices[i * 3 + 2] as usize
        } else {
            i * 3 + 2
        };
        let v0 = positions[idx0].truncate();
        let v1 = positions[idx1].truncate();
        let v2 = positions[idx2].truncate();

        let uv0 = tex_coords[idx0];
        let uv1 = tex_coords[idx1];
        let uv2 = tex_coords[idx2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let f = delta_uv1.y * delta_uv2.x - delta_uv1.x * delta_uv2.y;
        //let b = (delta_uv2.x * edge1 - delta_uv1.x * edge2) / f;
        let t = (delta_uv1.y * edge2 - delta_uv2.y * edge1) / f;
        let t = t.extend(0.0);

        tangents[idx0] = t;
        tangents[idx1] = t;
        tangents[idx2] = t;

        //bitangents[idx0] = b;
        //bitangents[idx1] = b;
        //bitangents[idx2] = b;
    }

    tangents
}
