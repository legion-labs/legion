use legion_data_offline::asset::AssetPathId;
use legion_data_runtime::{AssetDescriptor, AssetId};

use crate::{
    offline_data::{self, CompilableResource},
    runtime_data,
};

pub fn find_derived_path(path: &AssetPathId) -> AssetPathId {
    let offline_type = path.content_type();
    match offline_type {
        offline_data::Entity::TYPE_ID => path.push(runtime_data::Entity::TYPE),
        offline_data::Instance::TYPE_ID => path.push(runtime_data::Instance::TYPE),
        offline_data::Mesh::TYPE_ID => path.push(runtime_data::Mesh::TYPE),
        legion_graphics_offline::psd::TYPE_ID => path
            .push(legion_graphics_offline::texture::TYPE_ID)
            .push(legion_graphics_runtime::Texture::TYPE),
        legion_graphics_offline::material::TYPE_ID => {
            path.push(legion_graphics_runtime::Material::TYPE)
        }
        _ => {
            panic!("unrecognized offline type {}", offline_type);
        }
    }
}

pub fn find_derived_path_opt(path: &Option<AssetPathId>) -> Option<AssetId> {
    path.as_ref()
        .map(|path| find_derived_path(path).content_id())
}
