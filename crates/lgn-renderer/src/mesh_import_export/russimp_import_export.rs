use russimp::scene::Scene;
use lgn_math::Vec4;

use crate::static_mesh_render_data::StaticMeshRenderData;

pub struct AssimpWrapper {}

fn extract_vec3(ptr: *mut AiVector3D, num: usize, w: f32) -> Vec<Vec4> {
    let mut vectors = Vec::new();
    for i in 0..num {
        vectors.push( unsafe {
            let vector = *ptr.add(i as usize);
            Vec4::new(vector.x, vector.y, vector.z, w)
        });
    }
    vectors
}

fn print_scene_info(scene: &Scene) {
    println!("Scene info:");
    println!("Mesh count: {}", scene.num_meshes());
    println!("Animation count: {}", scene.num_animations());
    println!("Camera count: {}", scene.num_cameras());
    println!("Lights count: {}", scene.num_lights());
    println!("Materials count: {}", scene.num_materials());
    println!("Textures count: {}", scene.num_textures());
}

impl AssimpWrapper {
    pub fn new_mesh(path: String) -> Vec<StaticMeshRenderData> {
        let importer = Importer::new();
        let mut meshes = Vec::new();
        let scene = Scene::from_file("myfile.blend",
            vec![PostProcess::CalcTangentSpace,
                    PostProcess::Triangulate,
                    PostProcess::JoinIdenticalVertices,
                     PostProcess::SortByPType]).unwrap();

        if let Ok(scene) = importer.read_file(&path) {
            print_scene_info(&scene);
            for mesh in scene.mesh_iter() {
                let positions = extract_vec3(mesh.vertices, mesh.num_vertices() as usize, 1.0);
                let normals = extract_vec3(mesh.normals, mesh.num_vertices() as usize, 0.0);
                let mut indices = Vec::new();
                for face in mesh.face_iter() {
                    assert!(face.num_indices == 3);
                    for i in 0..face.num_indices {
                        indices.push(unsafe {*face.indices.add(i as usize)});
                    }
                }
                meshes.push(StaticMeshRenderData {
                    positions: Some(positions),
                    normals: Some(normals),
                    indices: Some(indices),
                    colors: None,
                    tex_coords: None
                })
            }
        } else {
            return Vec::new();
        }
        meshes
    }
}