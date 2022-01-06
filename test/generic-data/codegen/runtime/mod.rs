#[path = "../runtime/debug_cube.rs"]
mod debug_cube;
pub use debug_cube::*;

#[path = "../runtime/entity_dc.rs"]
mod entity_dc;
pub use entity_dc::*;

#[path = "../runtime/instance_dc.rs"]
mod instance_dc;
pub use instance_dc::*;

#[path = "../runtime/light_component.rs"]
mod light_component;
pub use light_component::*;

#[path = "../runtime/rotation_component.rs"]
mod rotation_component;
pub use rotation_component::*;

#[path = "../runtime/static_mesh_component.rs"]
mod static_mesh_component;
pub use static_mesh_component::*;

#[path = "../runtime/test_entity.rs"]
mod test_entity;
pub use test_entity::*;

#[path = "../runtime/transform_component.rs"]
mod transform_component;
pub use transform_component::*;

pub fn add_loaders(registry: &mut lgn_data_runtime::AssetRegistryOptions) {
    registry
        .add_loader_mut::<DebugCube>()
        .add_loader_mut::<EntityDc>()
        .add_loader_mut::<InstanceDc>()
        .add_loader_mut::<TestEntity>();
}
