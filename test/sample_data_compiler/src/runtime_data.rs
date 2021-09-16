use legion_data_runtime::{Asset, AssetId, AssetLoader, AssetRegistryOptions, AssetType};
use legion_math::prelude::*;
use serde::{Deserialize, Serialize};

pub trait CompilableAsset {
    const TYPE_ID: AssetType;
    type Creator: AssetLoader + Send + Default + 'static;
}

pub fn add_creators(mut registry: AssetRegistryOptions) -> AssetRegistryOptions {
    fn add_asset<T: CompilableAsset>(registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry.add_creator(T::TYPE_ID, Box::new(T::Creator::default()))
    }

    registry = add_asset::<Entity>(registry);
    registry = add_asset::<Instance>(registry);
    registry = add_asset::<Material>(registry);
    add_asset::<Mesh>(registry)
}

// ------------------ Entity -----------------------------------

#[derive(Asset, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub children: Vec<AssetId>,
    pub parent: Option<AssetId>,
    pub components: Vec<Box<dyn Component>>,
}

impl CompilableAsset for Entity {
    const TYPE_ID: AssetType = AssetType::new(b"runtime_entity");
    type Creator = EntityCreator;
}

#[derive(Default)]
pub struct EntityCreator {}

impl AssetLoader for EntityCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let deserialize: Result<Entity, Box<bincode::ErrorKind>> =
            bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}

#[typetag::serde]
pub trait Component: Send + Sync {}

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub apply_to_children: bool,
}

#[typetag::serde]
impl Component for Transform {}

#[derive(Serialize, Deserialize)]
pub struct Visual {
    pub renderable_geometry: String,
    pub shadow_receiver: bool,
    pub shadow_caster_sun: bool,
    pub shadow_caster_local: bool,
    pub gi_contribution: GIContribution,
}

#[typetag::serde]
impl Component for Visual {}

#[derive(Serialize, Deserialize)]
pub enum GIContribution {
    Default,
    Blocker,
    Exclude,
}

#[derive(Serialize, Deserialize)]
pub struct GlobalIllumination {}

#[typetag::serde]
impl Component for GlobalIllumination {}

#[derive(Serialize, Deserialize)]
pub struct NavMesh {
    pub voxelisation_config: VoxelisationConfig,
    pub layer_config: Vec<NavMeshLayerConfig>,
}

#[typetag::serde]
impl Component for NavMesh {}

#[derive(Serialize, Deserialize)]
pub struct VoxelisationConfig {}

#[derive(Serialize, Deserialize)]
pub struct NavMeshLayerConfig {}

#[derive(Serialize, Deserialize)]
pub struct View {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub projection_type: ProjectionType,
}

#[typetag::serde]
impl Component for View {}

#[derive(Serialize, Deserialize)]
pub enum ProjectionType {
    Orthogonal,
    Perspective,
}

#[derive(Serialize, Deserialize)]
pub struct Light {}

#[typetag::serde]
impl Component for Light {}

#[derive(Serialize, Deserialize)]
pub struct Physics {
    pub dynamic: bool,
    pub collision_geometry: String,
}

#[typetag::serde]
impl Component for Physics {}

// ------------------ Instance  -----------------------------------

#[derive(Asset, Serialize, Deserialize)]
pub struct Instance {
    pub original: Option<AssetId>,
}

impl CompilableAsset for Instance {
    const TYPE_ID: AssetType = AssetType::new(b"runtime_instance");
    type Creator = InstanceCreator;
}

#[derive(Default)]
pub struct InstanceCreator {}

impl AssetLoader for InstanceCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let deserialize: Result<Instance, Box<bincode::ErrorKind>> =
            bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}

// ------------------ Material -----------------------------------

#[derive(Asset, Serialize, Deserialize)]
pub struct Material {
    pub albedo: TextureReference,
    pub normal: TextureReference,
    pub roughness: TextureReference,
    pub metalness: TextureReference,
}

impl CompilableAsset for Material {
    const TYPE_ID: AssetType = AssetType::new(b"runtime_material");
    type Creator = MaterialCreator;
}

#[derive(Default)]
pub struct MaterialCreator {}

impl AssetLoader for MaterialCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let deserialize: Result<Material, Box<bincode::ErrorKind>> =
            bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}

pub type TextureReference = String;

// ------------------ Mesh -----------------------------------

#[derive(Asset, Serialize, Deserialize)]
pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

impl CompilableAsset for Mesh {
    const TYPE_ID: AssetType = AssetType::new(b"runtime_mesh");
    type Creator = MeshCreator;
}

#[derive(Default)]
pub struct MeshCreator {}

impl AssetLoader for MeshCreator {
    fn load(
        &mut self,
        _kind: AssetType,
        reader: &mut dyn std::io::Read,
    ) -> Result<Box<dyn Asset + Send + Sync>, std::io::Error> {
        let deserialize: Result<Mesh, Box<bincode::ErrorKind>> = bincode::deserialize_from(reader);
        match deserialize {
            Ok(asset) => Ok(Box::new(asset)),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err.to_string(),
            )),
        }
    }

    fn load_init(&mut self, _asset: &mut (dyn Asset + Send + Sync)) {}
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub material: Option<AssetId>,
}
