use gltf::{mesh::util::ReadIndices, Gltf};
use lgn_math::{Vec2, Vec3};

pub struct GltfWrapper {}

impl GltfWrapper {
    #[allow(clippy::never_loop)]
    pub fn new_mesh(path: String) -> (Vec<Vec3>, Vec<Vec3>, Vec<Vec2>, Vec<u32>) {
        let (gltf, buffers, _) = gltf::import(path).unwrap();

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        for mesh in gltf.meshes() {
            println!("Mesh #{:?}", mesh);
            for primitive in mesh.primitives() {
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
                                indices.push(idx as u32);
                            }
                        }
                        ReadIndices::U16(iter) => {
                            for idx in iter {
                                indices.push(idx as u32);
                            }
                        }
                        ReadIndices::U32(iter) => {
                            for idx in iter {
                                indices.push(idx);
                            }
                        }
                    }
                }

                return (positions, normals, uvs, indices);
            }
        }

        unimplemented!()
    }
}
