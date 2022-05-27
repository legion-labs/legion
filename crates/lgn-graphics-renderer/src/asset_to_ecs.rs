use std::sync::Arc;

use crate::components::{MaterialComponent, Mesh, ModelComponent, TextureComponent, TextureData};
use lgn_app::EventReader;
use lgn_asset_registry::AssetToEntityMap;
use lgn_data_runtime::{AssetRegistry, AssetRegistryEvent, Handle, ResourceDescriptor};
use lgn_ecs::prelude::{Commands, Entity, Res, ResMut};
use lgn_graphics_data::{
    runtime::{Material, Model},
    runtime_texture::Texture,
};
use lgn_tracing::info;

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(crate) fn process_load_events(
    asset_registry: Res<'_, Arc<AssetRegistry>>,
    mut asset_to_entity_map: ResMut<'_, AssetToEntityMap>,
    mut asset_loaded_events: EventReader<'_, '_, AssetRegistryEvent>,
    mut commands: Commands<'_, '_>,
) {
    for asset_loaded_event in asset_loaded_events.iter() {
        match asset_loaded_event {
            AssetRegistryEvent::AssetLoaded(resource) => match resource.id().kind {
                Texture::TYPE => {
                    crate::asset_to_ecs::create_texture(
                        Handle::<Texture>::from(resource.clone()),
                        &asset_registry,
                        &mut asset_to_entity_map,
                        &mut commands,
                    );
                }
                Material::TYPE => {
                    crate::asset_to_ecs::create_material(
                        Handle::<Material>::from(resource.clone()),
                        &asset_registry,
                        &mut asset_to_entity_map,
                        &mut commands,
                    );
                }
                Model::TYPE => {
                    crate::asset_to_ecs::create_model(
                        Handle::<Model>::from(resource.clone()),
                        &asset_registry,
                        &mut asset_to_entity_map,
                        &mut commands,
                    );
                }
                _ => {}
            },
        }
    }
}

pub(crate) fn create_material(
    asset_handle: Handle<Material>,
    asset_registry: &AssetRegistry,
    asset_to_entity_map: &mut AssetToEntityMap,
    commands: &mut Commands<'_, '_>,
) -> Option<Entity> {
    let material = asset_handle.get(asset_registry)?;

    let mut entity = if let Some(entity) = asset_to_entity_map.get(asset_handle.id()) {
        commands.entity(entity)
    } else {
        commands.spawn()
    };

    let asset_id = asset_handle.id();

    entity.insert(MaterialComponent::new(
        asset_handle,
        material.albedo.clone(),
        material.normal.clone(),
        material.metalness.clone(),
        material.roughness.clone(),
        material.sampler.clone(),
    ));

    info!(
        "Spawned {}: {} -> ECS id: {:?}",
        asset_id.kind.as_pretty().trim_start_matches("runtime_"),
        asset_id.id,
        entity.id(),
    );
    Some(entity.id())
}

pub(crate) fn create_texture(
    resource: Handle<Texture>,
    asset_registry: &AssetRegistry,
    asset_to_entity_map: &mut AssetToEntityMap,
    commands: &mut Commands<'_, '_>,
) -> Option<Entity> {
    let texture = resource.get(asset_registry)?;

    let mut entity = if let Some(entity) = asset_to_entity_map.get(resource.id()) {
        commands.entity(entity)
    } else {
        commands.spawn()
    };

    let texture_mips = texture // TODO: Avoid cloning in the future
        .texture_data
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();

    let texture_data = TextureData::from_slices(&texture_mips);

    let asset_id = resource.id();

    let texture_component = TextureComponent::new(
        resource,
        texture.width,
        texture.height,
        texture.format,
        texture.srgb,
        texture_data,
    );

    entity.insert(texture_component);
    info!(
        "Spawned {}: {} -> ECS id: {:?} | width: {}, height: {}, format: {:?}",
        asset_id.kind.as_pretty().trim_start_matches("runtime_"),
        asset_id.id,
        entity.id(),
        texture.width,
        texture.height,
        texture.format
    );
    Some(entity.id())
}

pub(crate) fn create_model(
    resource: Handle<Model>,
    asset_registry: &AssetRegistry,
    asset_to_entity_map: &mut AssetToEntityMap,
    commands: &mut Commands<'_, '_>,
) -> Option<Entity> {
    let model = resource.get(asset_registry)?;

    let mut entity = if let Some(entity) = asset_to_entity_map.get(resource.id()) {
        commands.entity(entity)
    } else {
        commands.spawn()
    };

    let mut meshes = Vec::new();
    for mesh in &model.meshes {
        meshes.push(Mesh {
            positions: mesh.positions.clone(),
            normals: if !mesh.normals.is_empty() {
                Some(mesh.normals.clone())
            } else {
                None
            },
            tangents: if !mesh.tangents.is_empty() {
                Some(mesh.tangents.clone())
            } else {
                None
            },
            tex_coords: if !mesh.tex_coords.is_empty() {
                Some(mesh.tex_coords.clone())
            } else {
                None
            },
            indices: if !mesh.indices.is_empty() {
                Some(mesh.indices.clone())
            } else {
                None
            },
            colors: if !mesh.colors.is_empty() {
                Some(mesh.colors.iter().map(|v| Into::into(*v)).collect())
            } else {
                None
            },
            material_id: mesh.material.clone(),
            bounding_sphere: Mesh::calculate_bounding_sphere(&mesh.positions),
        });
    }

    let asset_id = resource.id();

    let model_component = ModelComponent { resource, meshes };
    entity.insert(model_component);

    info!(
        "Spawned {}: {} -> ECS id: {:?}",
        asset_id.kind.as_pretty().trim_start_matches("runtime_"),
        asset_id.id,
        entity.id(),
    );
    Some(entity.id())
}
