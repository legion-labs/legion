use gltf::mesh::util::ReadIndices;
use lgn_math::{Vec3, Vec4};

use crate::static_mesh_render_data::StaticMeshRenderData;

pub struct GltfWrapper {}

impl GltfWrapper {
    pub fn new_mesh(path: String) -> Vec<StaticMeshRenderData> {
        let (gltf, buffers, _) = gltf::import(path).unwrap();

        let mut meshes = Vec::new();
        for mesh in gltf.meshes() {
            println!("Mesh #{:?}", mesh);
            for primitive in mesh.primitives() {
                let mut positions = Vec::new();
                let mut normals = Vec::new();
                let mut indices = Vec::new();

                println!("- Primitive #{}", primitive.index());
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
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

                meshes.push(StaticMeshRenderData {
                    positions: Some(
                        positions
                            .into_iter()
                            .map(|v: Vec3| Vec4::new(v.x, v.y, v.z, 1.0))
                            .collect(),
                    ),
                    normals: Some(
                        normals
                            .into_iter()
                            .map(|v: Vec3| Vec4::new(v.x, v.y, v.z, 0.0))
                            .collect(),
                    ),
                    tex_coords: None,
                    indices: Some(indices),
                    colors: None,
                });
            }
        }
        meshes
    }
}
