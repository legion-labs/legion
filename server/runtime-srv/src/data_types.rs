use legion_data_runtime::{Asset, AssetRegistryOptions, AssetType};
use legion_math::prelude::*;
use legion_utils::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Instance {
    original: String,
    overrides: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct Entity {
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
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

#[derive(Serialize, Deserialize, Default)]
struct Material {
    albedo: String,
    normal: String,
    roughness: String,
    metalness: String,
}

impl Asset for Material {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub const TEXT_MATERIAL: AssetType = AssetType::new(b"text_material");

fn load_material(
    _kind: legion_data_runtime::AssetType,
    reader: &mut dyn std::io::Read,
) -> Result<Box<dyn legion_data_runtime::Asset + Send + Sync>, std::io::Error> {
    let deserialize: Result<Material, ron::Error> = ron::de::from_reader(reader);
    match deserialize {
        Ok(material) => Ok(Box::new(material)),
        Err(_ron_err) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unable to read RON data",
        )),
    }
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

pub fn register_asset_loaders(asset_options: AssetRegistryOptions) -> AssetRegistryOptions {
    asset_options.add_creator(TEXT_MATERIAL, load_material)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_mesh_read_write() {
        let sub_mesh = SubMesh::default();
        let serialized = ron::ser::to_string(&sub_mesh).unwrap();
        dbg!(&serialized);
        let deserialized: SubMesh = ron::de::from_str(&serialized).unwrap();
        assert_eq!(sub_mesh, deserialized);
    }
}
