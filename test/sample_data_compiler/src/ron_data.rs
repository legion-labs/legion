use legion_math::prelude::*;
use legion_utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Instance {
    original: String,
    overrides: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    name: String,
    children: Vec<String>,
    parent: Option<String>,
    components: Vec<Component>,
}

#[derive(Serialize, Deserialize)]
enum GIContribution {
    Default,
    Blocker,
    Exclude,
}

#[derive(Serialize, Deserialize)]
struct Visual {
    renderable_geometry: String,
    shadow_receiver: bool,
    shadow_caster_sun: bool,
    shadow_caster_local: bool,
    gi_contribution: GIContribution,
}

#[derive(Serialize, Deserialize)]
struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    apply_to_children: bool,
}

#[derive(Serialize, Deserialize)]
struct GlobalIllumination {}

#[derive(Serialize, Deserialize)]
enum ProjectionType {
    Orthogonal,
    Perspective,
}

#[derive(Serialize, Deserialize)]
struct View {
    fov: f32,
    near: f32,
    far: f32,
    projection_type: ProjectionType,
}

#[derive(Serialize, Deserialize)]
struct Light {}

#[derive(Serialize, Deserialize)]
enum Component {
    Transform(Transform),
    Visual(Visual),
    GlobalIllumination(GlobalIllumination),
    Navmesh(NavMesh),
    View(View),
    Light(Light),
    Physics(Physics),
}

#[derive(Serialize, Deserialize)]
struct SubMesh {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    indices: Vec<u16>,
    material: String,
}

#[derive(Serialize, Deserialize)]
struct Mesh {
    sub_meshes: Vec<SubMesh>,
}

#[derive(Serialize, Deserialize)]
struct Material {
    albedo: String,
    normal: String,
    roughness: String,
    metalness: String,
}

#[derive(Serialize, Deserialize)]
struct Physics {
    dynamic: bool,
    collision_geometry: String,
}

#[derive(Serialize, Deserialize)]
struct Script {
    code: String,
    exposed_vars: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct CollisionMaterial {
    impact_script: Option<Script>,
}

#[derive(Serialize, Deserialize)]
struct VoxelisationConfig {}

#[derive(Serialize, Deserialize)]
struct NavMeshLayerConfig {}

#[derive(Serialize, Deserialize)]
struct NavMesh {
    voxelisation_config: VoxelisationConfig,
    layer_config: Vec<NavMeshLayerConfig>,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
    dependencies: Vec<String>,
    content_checksum: String,
}
