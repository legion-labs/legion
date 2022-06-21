use atlas::AtlasStorage;
use entity::EntityStorage;
use material::MaterialStorage;
use proto_salsa_compiler::{BuildParams, Locale, Platform, Target};

use crate::entity::EntityCompiler;
use crate::inputs::{Inputs, InputsStorage};
use crate::resource::ResourceCompiler;
use crate::texture::TextureStorage;
use crate::{
    atlas::AtlasCompiler, meta::MetaStorage, package::PackageCompiler, package::PackageStorage,
    resource::ResourceStorage,
};

mod atlas;
mod entity;
mod inputs;
mod material;
mod meta;
mod package;
mod resource;
mod texture;

#[salsa::database(
    InputsStorage,
    AtlasStorage,
    MaterialStorage,
    TextureStorage,
    PackageStorage,
    EntityStorage,
    MetaStorage,
    ResourceStorage
)]
#[derive(Default)]
pub struct DatabaseImpl {
    storage: salsa::Storage<Self>,
}

/// This impl tells salsa where to find the salsa runtime.
impl salsa::Database for DatabaseImpl {}

fn setup() -> DatabaseImpl {
    let mut db = DatabaseImpl::default();

    db.set_input_file(
        "TextureA.meta".to_string(),
        "Default:TextureA.jpg".to_string(),
    );
    db.set_input_file("TextureA.jpg".to_string(), "Texture A".to_string());

    db.set_input_file(
        "TextureB.meta".to_string(),
        "Default:TextureB.png".to_string(),
    );
    db.set_input_file("TextureB.png".to_string(), "Texture B".to_string());

    db.set_input_file(
        "TextureC.meta".to_string(),
        "English:TextureCEn.jpg\nFrench:TextureCFr.png".to_string(),
    );
    db.set_input_file(
        "TextureCEn.jpg".to_string(),
        "Texture in English".to_string(),
    );
    db.set_input_file(
        "TextureCFr.png".to_string(),
        "Texture en Français".to_string(),
    );

    db.set_input_file(
        "Atlas.entity".to_string(),
        "TextureA.meta,TextureB.meta,TextureC.meta".to_string(),
    );

    db.set_input_file(
        "MyWorld.entity".to_string(),
        "compile_atlas(Atlas.entity".to_string(),
    );

    db
}

fn main() {
    let db = setup();

    let build_params = BuildParams::new(Platform::PS5, Target::Client, Locale::English);

    let atlas_content = db.input_file("Atlas.entity".to_string());
    println!("Atlas: {}", db.compile_atlas(atlas_content, build_params));

    db.package_see_ps5();
}
