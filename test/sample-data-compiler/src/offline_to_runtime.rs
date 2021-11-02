use std::any::Any;

use legion_data_offline::ResourcePathId;
use legion_data_runtime::{Reference, Resource};
use sample_data_offline as offline_data;
use sample_data_runtime as runtime_data;

pub fn find_derived_path(path: &ResourcePathId) -> ResourcePathId {
    let offline_type = path.content_type();
    match offline_type {
        offline_data::Entity::TYPE => path.push(runtime_data::Entity::TYPE),
        offline_data::Instance::TYPE => path.push(runtime_data::Instance::TYPE),
        offline_data::Mesh::TYPE => path.push(runtime_data::Mesh::TYPE),
        legion_graphics_offline::PsdFile::TYPE => path
            .push(legion_graphics_offline::Texture::TYPE)
            .push(legion_graphics_runtime::Texture::TYPE),
        legion_graphics_offline::Material::TYPE => {
            path.push(legion_graphics_runtime::Material::TYPE)
        }
        _ => {
            panic!("unrecognized offline type {}", offline_type);
        }
    }
}

pub fn to_reference<T>(path: &Option<ResourcePathId>) -> Option<Reference<T>>
where
    T: Any + Resource,
{
    path.as_ref()
        .map(|path| Reference::Passive(path.content_id()))
}
