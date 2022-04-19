use std::collections::HashMap;

use lgn_data_runtime::prelude::*;
use lgn_ecs::prelude::Commands;
use lgn_hierarchy::prelude::{BuildChildren, Parent};
use lgn_tracing::{info, warn};

use crate::ResourceMetaInfo;

pub struct SceneInstance {
    root_resource: ResourceTypeAndId,
    _root_handle: Handle<sample_data::runtime::Entity>,
    id_to_entity_map: HashMap<ResourceTypeAndId, lgn_ecs::entity::Entity>,
    //entity_to_id: HashMap<lgn_ecs::entity::Entity, ResourceTypeAndId>,
}

impl SceneInstance {
    pub(crate) fn new(
        root_resource: ResourceTypeAndId,
        root_handle: Handle<sample_data::runtime::Entity>,
    ) -> Self {
        Self {
            root_resource,
            _root_handle: root_handle,
            id_to_entity_map: HashMap::new(),
        }
    }

    pub(crate) fn find_entity(
        &self,
        resource_id: &ResourceTypeAndId,
    ) -> Option<&lgn_ecs::entity::Entity> {
        self.id_to_entity_map.get(resource_id)
    }

    pub(crate) fn unspawn_all(&mut self, commands: &mut Commands<'_, '_>) {
        for (_id, entity) in self.id_to_entity_map.drain() {
            commands.entity(entity).despawn();
        }
    }

    pub(crate) fn spawn_entity_hierarchy(
        &mut self,
        entity: Handle<sample_data::runtime::Entity>,
        asset_registry: &AssetRegistry,
        commands: &mut Commands<'_, '_>,
    ) {
        let mut queue = vec![entity];
        while let Some(handle) = queue.pop() {
            let runtime_entity = handle.get();
            if runtime_entity.is_none() {
                if handle.id().kind != sample_data::runtime::Instance::TYPE {
                    warn!(
                        "Failed to spawn {:?}. Resource not found in AssetRegistry ",
                        handle.id()
                    );
                }
                continue;
            }
            let resource_id = handle.id();
            let runtime_entity = runtime_entity.unwrap();

            // Ignore entity with parent that don't exists yet
            // The parent will spawn the children entities when it's available
            if self.root_resource != resource_id {
                if let Some(parent) = &runtime_entity.parent {
                    if self.id_to_entity_map.get(&parent.id()).is_none() {
                        continue;
                    }
                }
            }

            let entity_id = self
                .id_to_entity_map
                .get(&resource_id)
                .copied()
                .unwrap_or_else(|| {
                    let entity_id = commands.spawn().id();
                    self.id_to_entity_map.insert(resource_id, entity_id);
                    entity_id
                });

            // Look at the existing Ecs Entity children and unspawn
            // the children not present in the data anymore
            /*if let Ok(existing_children) = entity_with_children_query.get(entity_id) {
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
            }*/

            for children in &runtime_entity.children {
                if let Some(handle) = children.get_active_handle() {
                    queue.push(handle);
                }
            }

            let children = runtime_entity
                .children
                .iter()
                .rev()
                .filter_map(|child_ref| {
                    if let Some(child) = child_ref.get_active_handle() {
                        let child_res_id = child.id();
                        queue.push(child);
                        Some(
                            self.id_to_entity_map
                                .get(&child_res_id)
                                .copied()
                                .unwrap_or_else(|| {
                                    let entity_id = commands.spawn().id();
                                    self.id_to_entity_map.insert(child_res_id, entity_id);
                                    entity_id
                                }),
                        )
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let mut entity_command = commands.entity(entity_id);
            entity_command.insert(ResourceMetaInfo { id: handle.id() });
            entity_command.insert(lgn_core::Name::new(resource_id.id.to_string()));

            if !children.is_empty() {
                entity_command.push_children(children.as_slice());
            }

            for component in &runtime_entity.components {
                if let Some(name) = component.downcast_ref::<sample_data::runtime::Name>() {
                    info!(
                        "Spawning Entity {:?}|{}|{}",
                        entity_id, resource_id.id, &name.name
                    );
                }

                if let Some(installer) = asset_registry.get_component_installer(component.type_id())
                {
                    if let Err(err) =
                        installer.install_component(component.as_ref(), &mut entity_command)
                    {
                        lgn_tracing::error!("Failed to install component: {}", err);
                    }
                }
            }

            /*for component in &runtime_entity.components {
                if let Some(visual) = component.downcast_ref::<runtime_data::Visual>() {
                    entity.insert(VisualComponent::new(
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
                            sample_data::LightType::Omnidirectional => LightType::OmniDirectional,
                            sample_data::LightType::Directional => LightType::Directional,
                            sample_data::LightType::Spotlight => LightType::Spot,
                            _ => unreachable!("Unrecognized light type"),
                        },
                        color: light.color,
                        radiance: light.radiance,
                        cone_angle: light.cone_angle,
                        enabled: light.enabled,
                        ..LightComponent::default()
                    });
                } else if let Some(_gltf_loader) =
                    component.downcast_ref::<runtime_data::GltfLoader>()
                {
                    // nothing to do
                }
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
            }*/

            if let Some(parent) = runtime_entity.parent.as_ref() {
                if let Some(parent_ecs_entity) = self.id_to_entity_map.get(&parent.id()) {
                    entity_command.insert(Parent(*parent_ecs_entity));
                }
            }
        }
    }
}
