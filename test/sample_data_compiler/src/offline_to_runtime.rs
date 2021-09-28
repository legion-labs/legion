use std::convert::TryFrom;

use legion_data_offline::{asset::AssetPathId, resource::ResourceType};
use legion_data_runtime::AssetId;

use crate::{
    offline_data::{self, CompilableResource},
    runtime_data::{self, CompilableAsset},
};

pub fn convert_offline_to_content_path(path: &AssetPathId) -> AssetPathId {
    let offline_type = ResourceType::try_from(path.content_type()).ok().unwrap();
    let runtime_type = match offline_type {
        offline_data::Entity::TYPE_ID => runtime_data::Entity::TYPE_ID.content(),
        offline_data::Instance::TYPE_ID => runtime_data::Instance::TYPE_ID.content(),
        offline_data::Mesh::TYPE_ID => runtime_data::Mesh::TYPE_ID.content(),
        legion_graphics_offline::psd::TYPE_ID => {
            legion_graphics_offline::texture::TYPE_ID.content()
        }
        legion_graphics_offline::material::TYPE_ID => {
            legion_graphics_runtime::material::TYPE_ID.content()
        }
        _ => {
            panic!("unrecognized offline type {}", offline_type.content());
        }
    };
    path.push(runtime_type)
}

pub fn convert_offline_path_to_runtime_id(path: &AssetPathId) -> Option<AssetId> {
    let path = convert_offline_to_content_path(path);
    AssetId::try_from(path.content_id()).ok()
}

pub fn convert_optional_offline_path_to_runtime_id(path: &Option<AssetPathId>) -> Option<AssetId> {
    if let Some(path) = path {
        convert_offline_path_to_runtime_id(path)
    } else {
        None
    }
}
