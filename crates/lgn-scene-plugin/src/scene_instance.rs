use lgn_asset_registry::AssetToEntityMap;
use lgn_core::Name;
use lgn_data_runtime::{AssetRegistry, Resource, ResourceTypeAndId};
use lgn_ecs::prelude::{Commands, Query};
use lgn_graphics_data::runtime::ModelReferenceType;
use lgn_graphics_renderer::{
    components::{LightComponent, LightType, VisualComponent},
    features::mesh_feature::MeshRenderObjectSet,
};
use lgn_hierarchy::prelude::{BuildChildren, Children, Parent};
use lgn_tracing::{error, info, warn};
use lgn_transform::components::{GlobalTransform, Transform};
use sample_data::runtime as runtime_data;

pub struct SceneInstance {
    pub root_resource: ResourceTypeAndId,
    pub asset_to_entity_map: AssetToEntityMap,
}

impl SceneInstance {}

impl SceneInstance {
    pub(crate) fn new(root_resource: ResourceTypeAndId) -> Self {
        Self {
            root_resource,
            asset_to_entity_map: AssetToEntityMap::default(),
        }
    }

    pub(crate) fn spawn_entity_hierarchy(
        &mut self,
        resource_id: ResourceTypeAndId,
        asset_registry: &AssetRegistry,
        commands: &mut Commands<'_, '_>,
        entity_with_children_query: &Query<'_, '_, &Children>,
        tmp_mesh_set: &mut MeshRenderObjectSet,
    ) {
        let mut queue = vec![resource_id];

        while let Some(resource_id) = queue.pop() {
            let runtime_entity = asset_registry
                .get_untyped(resource_id)
                .and_then(|handle| handle.get::<sample_data::runtime::Entity>(asset_registry));

            if runtime_entity.is_none() {
                warn!(
                    "Failed to spawn {:?}. Resource not found in AssetRegistry ",
                    resource_id
                );
                continue;
            }
            let runtime_entity = runtime_entity.unwrap();

            // Ignore entity with parent that don't exists yet
            // The parent will spawn the children entities when it's available
            if self.root_resource != resource_id {
                if let Some(parent) = &runtime_entity.parent {
                    if self.asset_to_entity_map.get(parent.id()).is_none() {
                        continue;
                    }
                }
            }

            let entity_id = self
                .asset_to_entity_map
                .get(resource_id)
                .unwrap_or_else(|| {
                    let entity_id = commands.spawn().id();
                    self.asset_to_entity_map.insert(resource_id, entity_id);
                    entity_id
                });

            // Look at the existing Ecs Entity children and unspawn
            // the children not present in the data anymore
            if let Ok(existing_children) = entity_with_children_query.get(entity_id) {
                for previous_child in existing_children.iter() {
                    if let Some(resource_id) =
                        self.asset_to_entity_map.get_resource_id(*previous_child)
                    {
                        if runtime_entity
                            .children
                            .iter()
                            .find(|child_ref| child_ref.id() == resource_id)
                            == None
                        {
                            commands.entity(*previous_child).despawn();
                            self.asset_to_entity_map.remove(*previous_child);
                        }
                    }
                }
            }

            let children = runtime_entity
                .children
                .iter()
                .rev()
                .filter_map(|child_ref| {
                    let child_res_id = child_ref.id();
                    if child_res_id.kind == sample_data::runtime::Entity::TYPE {
                        queue.push(child_res_id);
                        Some(
                            self.asset_to_entity_map
                                .get(child_res_id)
                                .unwrap_or_else(|| {
                                    let entity_id = commands.spawn().id();
                                    self.asset_to_entity_map.insert(child_res_id, entity_id);
                                    entity_id
                                }),
                        )
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let mut entity = commands.entity(entity_id);
            if !children.is_empty() {
                entity.push_children(children.as_slice());
            }

            let mut local_transform: Option<Transform> = None;
            let mut entity_name: Option<String> = None;

            for component in &runtime_entity.components {
                if let Some(transform) = component.downcast_ref::<runtime_data::Transform>() {
                    local_transform = Some(Transform {
                        translation: transform.position,
                        rotation: transform.rotation,
                        scale: transform.scale,
                    });
                } else if let Some(script) =
                    component.downcast_ref::<lgn_scripting::runtime::ScriptComponent>()
                {
                    entity.insert(script.clone());
                } else if let Some(name) = component.downcast_ref::<runtime_data::Name>() {
                    entity_name = Some(name.name.clone());
                } else if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
                    entity.insert(VisualComponent::new(
                        tmp_mesh_set,
                        visual
                            .renderable_geometry
                            .as_ref()
                            .map(ModelReferenceType::id),
                        visual.color,
                        visual.color_blend,
                    ));
                } else if let Some(gi) =
                    component.downcast_ref::<runtime_data::GlobalIllumination>()
                {
                    entity.insert(gi.clone());
                } else if let Some(nav_mesh) = component.downcast_ref::<runtime_data::NavMesh>() {
                    entity.insert(nav_mesh.clone());
                } else if let Some(view) = component.downcast_ref::<runtime_data::View>() {
                    entity.insert(view.clone());
                } else if let Some(light) = component.downcast_ref::<runtime_data::Light>() {
                    entity.insert(LightComponent {
                        light_type: match light.light_type {
                            sample_data::LightType::Omnidirectional => LightType::Omnidirectional,
                            sample_data::LightType::Directional => LightType::Directional,
                            sample_data::LightType::Spotlight => LightType::Spotlight {
                                cone_angle: light.cone_angle,
                            },
                            _ => unreachable!("Unrecognized light type"),
                        },
                        color: light.color,
                        radiance: light.radiance,
                        enabled: light.enabled,
                        ..LightComponent::default()
                    });
                } else if let Some(_gltf_loader) =
                    component.downcast_ref::<runtime_data::GltfLoader>()
                {
                    // nothing to do
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidBox>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidCapsule>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidConvexMesh>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidHeightField>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidPlane>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidSphere>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsRigidTriangleMesh>()
                {
                    entity.insert(physics.clone());
                } else if let Some(physics_settings) =
                    component.downcast_ref::<lgn_physics::runtime::PhysicsSceneSettings>()
                {
                    entity.insert(physics_settings.clone());
                } else if let Some(camera_setup) =
                    component.downcast_ref::<lgn_graphics_data::runtime::CameraSetup>()
                {
                    entity.insert(camera_setup.clone());
                } else {
                    error!(
                        "Unhandle component type {} in entity {}",
                        component.get_type().get_type_name(),
                        resource_id,
                    );
                }
            }

            if let Some(parent) = runtime_entity.parent.as_ref() {
                if let Some(parent_ecs_entity) = self.asset_to_entity_map.get(parent.id()) {
                    entity.insert(Parent(parent_ecs_entity));
                }
            }

            let name = entity_name.get_or_insert(resource_id.id.to_string());
            entity.insert(Name::new(name.clone()));
            entity.insert(local_transform.unwrap_or_default());
            entity.insert(GlobalTransform::identity());

            info!(
                "Spawned Entity: {} -> ECS id: {:?}| {}",
                resource_id.id, entity_id, name,
            );
        }
    }
}
