use gltf::mesh::util::{ReadIndices, ReadTexCoords};
use lgn_math::{Vec3, Vec4, Vec2};

use crate::static_mesh_render_data::{StaticMeshRenderData, calculate_tangents};

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
                let mut tex_coords = Vec::new();
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
                if let Some(tex_coords_option) = reader.read_tex_coords(0) {
                    match tex_coords_option {
                        ReadTexCoords::F32(iter) => {
                            for tex_coord in iter {
                                tex_coords.push(Vec2::new(tex_coord[0], tex_coord[1]));
                            }
                        },
                        _ => unreachable!("Integer UVs are not supported")
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
                meshes.push(StaticMeshRenderData {
                    positions: Some(positions),
                    normals: Some(normals),
                    tangents: Some(tangents),
                    tex_coords: Some(tex_coords),
                    indices,
                    colors: None,
                });
            }
        }
        meshes
    }
}
