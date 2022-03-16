use crate::{
    components::{Mesh, ModelComponent, TextureComponent, TextureData},
    MaterialComponent,
};
use lgn_asset_registry::AssetToEntityMap;
use lgn_data_runtime::{AssetRegistry, ResourceTypeAndId};
use lgn_ecs::prelude::{Commands, Entity};
use lgn_tracing::info;

pub(crate) fn create_material(
    asset_id: &ResourceTypeAndId,
    asset_registry: &AssetRegistry,
    asset_to_entity_map: &mut AssetToEntityMap,
    commands: &mut Commands<'_, '_>,
) -> Option<Entity> {
    let material = asset_registry
        .get_untyped(*asset_id)
        .and_then(|handle| handle.get::<lgn_graphics_data::runtime::Material>(asset_registry))?;

    let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
        commands.entity(entity)
    } else {
        commands.spawn()
    };

    entity.insert(MaterialComponent::new(
        *asset_id,
        material.albedo.clone(),
        material.normal.clone(),
        material.metalness.clone(),
        material.roughness.clone(),
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
    asset_id: &ResourceTypeAndId,
    asset_registry: &AssetRegistry,
    asset_to_entity_map: &mut AssetToEntityMap,
    commands: &mut Commands<'_, '_>,
) -> Option<Entity> {
    let texture = asset_registry.get_untyped(*asset_id).and_then(|handle| {
        handle.get::<lgn_graphics_data::runtime_texture::Texture>(asset_registry)
    })?;

    let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
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

    let texture_component = TextureComponent::new(
        *asset_id,
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
    asset_id: &ResourceTypeAndId,
    asset_registry: &AssetRegistry,
    asset_to_entity_map: &mut AssetToEntityMap,
    commands: &mut Commands<'_, '_>,
) -> Option<Entity> {
    let model = asset_registry
        .get_untyped(*asset_id)
        .and_then(|handle| handle.get::<lgn_graphics_data::runtime::Model>(asset_registry))?;

    let mut entity = if let Some(entity) = asset_to_entity_map.get(*asset_id) {
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
    let model_component = ModelComponent {
        model_id: *asset_id,
        meshes,
    };
    entity.insert(model_component);

    info!(
        "Spawned {}: {} -> ECS id: {:?}",
        asset_id.kind.as_pretty().trim_start_matches("runtime_"),
        asset_id.id,
        entity.id(),
    );
    Some(entity.id())
}
