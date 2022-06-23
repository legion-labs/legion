use crate::{
    atlas::AtlasStorage, collision::CollisionStorage, entity::EntityStorage,
    expression::ExpressionStorage, inputs::InputsStorage, material::MaterialStorage,
    meta::MetaStorage, navmesh::NavmeshStorage, package::PackageStorage,
    runtime_dependency::RuntimeDependencyStorage, texture::TextureStorage,
};

#[salsa::database(
    ExpressionStorage,
    InputsStorage,
    AtlasStorage,
    MaterialStorage,
    TextureStorage,
    PackageStorage,
    EntityStorage,
    MetaStorage,
    RuntimeDependencyStorage,
    NavmeshStorage,
    CollisionStorage
)]
#[derive(Default)]
pub struct DatabaseImpl {
    storage: salsa::Storage<Self>,
}

/// This impl tells salsa where to find the salsa runtime.
impl salsa::Database for DatabaseImpl {}
