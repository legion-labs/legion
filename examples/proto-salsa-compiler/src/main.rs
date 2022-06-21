use atlas::AtlasStorage;
use entity::EntityStorage;
use material::MaterialStorage;
use proto_salsa_compiler::{BuildParams, Locale, Platform, Target};

use crate::inputs::{Inputs, InputsStorage};
use crate::texture::TextureStorage;
use crate::{atlas::AtlasCompiler, meta::MetaStorage, package::PackageStorage};

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
    MetaStorage
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
        "TextureA.meta".to_owned(),
        "Default:TextureA.jpg".to_owned(),
    );
    db.set_input_file("TextureA.jpg".to_owned(), "Texture A".to_owned());

    db.set_input_file(
        "TextureB.meta".to_owned(),
        "Default:TextureB.png".to_owned(),
    );
    db.set_input_file("TextureB.png".to_owned(), "Texture B".to_owned());

    db.set_input_file(
        "TextureC.meta".to_owned(),
        "English:TextureCEn.jpg\nFrench:TextureCFr.png".to_owned(),
    );
    db.set_input_file("TextureCEn.jpg".to_owned(), "Texture in English".to_owned());
    db.set_input_file(
        "TextureCFr.jpg".to_owned(),
        "Texture en Français".to_owned(),
    );

    db.set_input_file(
        "Atlas.entity".to_owned(),
        "TextureA.meta,TextureB.meta,TextureC.meta".to_owned(),
    );

    db
}

fn main() {
    let db = setup();

    let build_params = BuildParams::new(Platform::PS5, Target::Client, Locale::English);

    println!(
        "Atlas: {}",
        db.compile_atlas("Atlas.entity".to_owned(), build_params)
    );
}
