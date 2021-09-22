// Types that will eventually moved to library crates

use std::any::{Any, TypeId};

use legion_data_offline::{
    asset::AssetPathId,
    resource::{Resource, ResourceId, ResourceProcessor, ResourceType},
};
use legion_math::prelude::*;
use serde::{Deserialize, Serialize};

pub trait CompilableResource {
    const TYPE_ID: ResourceType;
    type Processor: ResourceProcessor + Default + 'static;
}

// ------------------ Entity -----------------------------------

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub children: Vec<ResourceId>,
    pub parent: Option<ResourceId>,
    pub components: Vec<Box<dyn Component>>,
}

impl CompilableResource for Entity {
    const TYPE_ID: ResourceType = ResourceType::new(b"offline_entity");
    type Processor = EntityProcessor;
}

#[derive(Default)]
pub struct EntityProcessor {}

impl ResourceProcessor for EntityProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Entity::default())
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
        Vec::new()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Entity>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: Entity = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }
}

#[typetag::serde]
pub trait Component: Any {}

/// Note: Based on impl of dyn Any
impl dyn Component {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Component>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Component>(&self) -> Option<&T> {
        if self.is::<T>() {
            #[allow(unsafe_code)]
            unsafe {
                Some(&*((self as *const dyn Component).cast::<T>()))
            }
        } else {
            None
        }
    }
}

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

#[derive(Resource, Serialize, Deserialize)]
pub struct Instance {
    pub original: Option<ResourceId>,
}

impl CompilableResource for Instance {
    const TYPE_ID: ResourceType = ResourceType::new(b"offline_instance");
    type Processor = InstanceProcessor;
}

#[derive(Default)]
pub struct InstanceProcessor {}

impl ResourceProcessor for InstanceProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Instance { original: None })
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
        Vec::new()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Instance>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: Instance = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }
}

// ------------------ Material -----------------------------------

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct Material {
    pub albedo: TextureReference,
    pub normal: TextureReference,
    pub roughness: TextureReference,
    pub metalness: TextureReference,
}

impl CompilableResource for Material {
    const TYPE_ID: ResourceType = ResourceType::new(b"offline_material");
    type Processor = MaterialProcessor;
}

#[derive(Default)]
pub struct MaterialProcessor {}

impl ResourceProcessor for MaterialProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Material::default())
    }

    fn extract_build_dependencies(&mut self, _resource: &dyn Resource) -> Vec<AssetPathId> {
        // let material = resource.downcast_ref::<Material>().unwrap();
        Vec::new()
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Material>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: Material = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }
}

pub type TextureReference = String;

// ------------------ Mesh -----------------------------------

#[derive(Resource, Serialize, Deserialize)]
pub struct Mesh {
    pub sub_meshes: Vec<SubMesh>,
}

impl CompilableResource for Mesh {
    const TYPE_ID: ResourceType = ResourceType::new(b"offline_mesh");
    type Processor = MeshProcessor;
}

#[derive(Default)]
pub struct MeshProcessor {}

impl ResourceProcessor for MeshProcessor {
    fn new_resource(&mut self) -> Box<dyn Resource> {
        Box::new(Mesh {
            sub_meshes: Vec::default(),
        })
    }

    fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId> {
        let mesh = resource.downcast_ref::<Mesh>().unwrap();
        let mut material_refs: Vec<AssetPathId> = mesh
            .sub_meshes
            .iter()
            .filter_map(|sub_mesh| sub_mesh.material.as_ref())
            .cloned()
            .collect();
        material_refs.sort();
        material_refs.dedup();
        material_refs
    }

    fn write_resource(
        &mut self,
        resource: &dyn Resource,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let resource = resource.downcast_ref::<Mesh>().unwrap();
        serde_json::to_writer(writer, resource).unwrap();
        Ok(1) // no bytes written exposed by serde.
    }

    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Resource>> {
        let resource: Mesh = serde_json::from_reader(reader).unwrap();
        Ok(Box::new(resource))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub material: Option<AssetPathId>,
}
