use std::{any::Any, io, path::Path};

use gltf::{
    mesh::util::{ReadIndices, ReadTexCoords},
    Document, Gltf,
};
use lgn_math::{Vec2, Vec3, Vec4};
//use crate::static_mesh_render_data::{StaticMeshRenderData, calculate_tangents};

use lgn_data_offline::resource::{OfflineResource, ResourceProcessor, ResourceProcessorError};
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};

#[resource("gltf")]
#[derive(Default)]
pub struct GltfFile {
    pub document: Option<Document>,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

impl GltfFile {
    pub fn from_path(path: &Path) -> GltfFile {
        let (document, buffers, images) = gltf::import(path).unwrap();
        Self {
            document: Some(document),
            buffers,
            images,
        }
    }

    pub fn new_mesh(&self) {
        //path: String) -> Vec<StaticMeshRenderData> {
        //let mut meshes = Vec::new();
        for mesh in self.document.as_ref().unwrap().meshes() {
            println!("Mesh #{:?}", mesh);
            for primitive in mesh.primitives() {
                let mut positions: Vec<Vec3> = Vec::new();
                let mut normals: Vec<Vec3> = Vec::new();
                let mut tex_coords: Vec<Vec2> = Vec::new();
                let mut indices: Vec<u32> = Vec::new();

                println!("- Primitive #{}", primitive.index());
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

                //let positions = positions
                //    .into_iter()
                //    .map(|v: Vec3| Vec4::new(v.x, v.y, v.z, 1.0))
                //    .collect::<Vec<Vec4>>();
                //let normals = normals
                //    .into_iter()
                //    .map(|v: Vec3| Vec4::new(v.x, v.y, v.z, 0.0))
                //    .collect();
                //let indices = Some(indices);
                //let tangents = calculate_tangents(&positions, &tex_coords, &indices);
                //meshes.push(StaticMeshRenderData {
                //    positions: Some(positions),
                //    normals: Some(normals),
                //    tangents: Some(tangents),
                //    tex_coords: Some(tex_coords),
                //    indices,
                //    colors: None,
                //});
            }
        }
        //meshes
    }
}

impl Asset for GltfFile {
    type Loader = GltfFileProcessor;
}

impl OfflineResource for GltfFile {
    type Processor = GltfFileProcessor;
}

fn read_usize(reader: &mut dyn io::Read) -> io::Result<usize> {
    let mut byte_size = 0usize.to_ne_bytes();
    reader.read_exact(&mut byte_size)?;
    Ok(usize::from_ne_bytes(byte_size))
}

fn write_usize(writer: &mut dyn std::io::Write, v: usize) -> io::Result<usize> {
    let bytes = v.to_ne_bytes();
    writer.write_all(&bytes)?;
    Ok(bytes.len())
}

fn read_usize_and_buffer(reader: &mut dyn io::Read) -> io::Result<Vec<u8>> {
    let size = read_usize(reader)?;
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_usize_and_buffer(writer: &mut dyn std::io::Write, v: &[u8]) -> io::Result<usize> {
    let written = write_usize(writer, v.len())?;
    writer.write_all(v)?;
    Ok(written + v.len())
}

#[derive(Default)]
pub struct GltfFileProcessor {}

impl AssetLoader for GltfFileProcessor {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let mut document_bytes = read_usize_and_buffer(reader)?;
        let buffers_length = read_usize(reader)?;
        let mut buffers = Vec::new();
        for i in 0..buffers_length {
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

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {}
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
