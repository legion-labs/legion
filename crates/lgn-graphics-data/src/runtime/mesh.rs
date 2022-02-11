use std::{any::Any, io};

use lgn_math::{Vec2, Vec3, Vec4};
//use crate::static_mesh_render_data::{StaticMeshRenderData, calculate_tangents};

use lgn_data_offline::resource::{OfflineResource, ResourceProcessor, ResourceProcessorError};
use lgn_data_runtime::{resource, Asset, AssetLoader, AssetLoaderError, Resource};

use crate::helpers::{
    read_vec_u32, read_vec_vec2, read_vec_vec4, write_usize, write_vec_u32, write_vec_vec2,
    write_vec_vec4,
};

#[resource("test_mesh")]
#[derive(Default)]
pub struct Mesh {
    pub positions: Option<Vec<Vec4>>,
    pub normals: Option<Vec<Vec4>>,
    pub tangents: Option<Vec<Vec4>>,
    pub tex_coords: Option<Vec<Vec2>>,
    pub indices: Option<Vec<u32>>,
    pub colors: Option<Vec<Vec4>>,
}

impl Asset for Mesh {
    type Loader = MeshProcessor;
}

impl OfflineResource for Mesh {
    type Processor = MeshProcessor;
}

#[derive(Default)]
pub struct MeshProcessor {}

impl AssetLoader for MeshProcessor {
    fn load(
        &mut self,
        reader: &mut dyn io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, AssetLoaderError> {
        let positions = read_vec_vec4(reader)?;
        let normals = read_vec_vec4(reader)?;
        let tangents = read_vec_vec4(reader)?;
        let tex_coords = read_vec_vec2(reader)?;
        let indices = read_vec_u32(reader)?;
        let colors = read_vec_vec4(reader)?;
        Ok(Box::new(Mesh {
            positions: if positions.is_empty() {
                None
            } else {
                Some(positions)
            },
            normals: if normals.is_empty() {
                None
            } else {
                Some(normals)
            },
            tangents: if tangents.is_empty() {
                None
            } else {
                Some(tangents)
            },
            tex_coords: if tex_coords.is_empty() {
                None
            } else {
                Some(tex_coords)
            },
            indices: if indices.is_empty() {
                None
            } else {
                Some(indices)
            },
            colors: if colors.is_empty() {
                None
            } else {
                Some(colors)
            },
        }))
    }

    fn load_init(&mut self, asset: &mut (dyn Any + Send + Sync)) {}
}

impl ResourceProcessor for MeshProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(Mesh::default())
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
        let mesh = resource.downcast_ref::<Mesh>().unwrap();
        let mut written = 0;
        written += if let Some(positions) = mesh.positions.as_ref() {
            write_vec_vec4(writer, positions)?
        } else {
            write_usize(writer, 0)?
        };
        written += if let Some(normals) = mesh.normals.as_ref() {
            write_vec_vec4(writer, normals)?
        } else {
            write_usize(writer, 0)?
        };
        written += if let Some(tangents) = mesh.tangents.as_ref() {
            write_vec_vec4(writer, tangents)?
        } else {
            write_usize(writer, 0)?
        };
        written += if let Some(tex_coords) = mesh.tex_coords.as_ref() {
            write_vec_vec2(writer, tex_coords)?
        } else {
            write_usize(writer, 0)?
        };
        written += if let Some(indices) = mesh.indices.as_ref() {
            write_vec_u32(writer, indices)?
        } else {
            write_usize(writer, 0)?
        };
        written += if let Some(colors) = mesh.colors.as_ref() {
            write_vec_vec4(writer, colors)?
        } else {
            write_usize(writer, 0)?
        };
        Ok(written)
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
        Ok(self.load(reader)?)
    }
}
