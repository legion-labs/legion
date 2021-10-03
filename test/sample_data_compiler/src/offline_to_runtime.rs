use legion_data_offline::ResourcePathId;
use legion_data_runtime::{Resource, ResourceId};

use crate::{
    offline_data::{self},
    runtime_data,
};

pub fn find_derived_path(path: &ResourcePathId) -> ResourcePathId {
    let offline_type = path.content_type();
    match offline_type {
        offline_data::Entity::TYPE => path.push(runtime_data::Entity::TYPE),
        offline_data::Instance::TYPE => path.push(runtime_data::Instance::TYPE),
        offline_data::Mesh::TYPE => path.push(runtime_data::Mesh::TYPE),
        legion_graphics_offline::psd::PsdFile::TYPE => path
            .push(legion_graphics_offline::texture::Texture::TYPE)
            .push(legion_graphics_runtime::Texture::TYPE),
        legion_graphics_offline::material::Material::TYPE => {
            path.push(legion_graphics_runtime::Material::TYPE)
        }
        _ => {
            panic!("unrecognized offline type {}", offline_type);
        }
    }
}

pub fn find_derived_path_opt(path: &Option<ResourcePathId>) -> Option<ResourceId> {
    path.as_ref()
        .map(|path| find_derived_path(path).content_id())
}
