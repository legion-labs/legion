use atlas::AtlasStorage;
use entity::EntityStorage;
use material::MaterialStorage;
use proto_salsa_compiler::{Locale, Platform, Target};

use crate::inputs::{Inputs, InputsStorage};
use crate::texture::TextureStorage;
use crate::{atlas::AtlasCompiler, material::MaterialCompiler, package::PackageStorage};

mod atlas;
mod entity;
mod inputs;
mod material;
mod package;
mod resource;
mod texture;

#[salsa::database(
    InputsStorage,
    AtlasStorage,
    MaterialStorage,
    TextureStorage,
    PackageStorage,
    EntityStorage
)]
#[derive(Default)]
pub struct DatabaseImpl {
    storage: salsa::Storage<Self>,
}

/// This impl tells salsa where to find the salsa runtime.
impl salsa::Database for DatabaseImpl {}

fn setup() -> DatabaseImpl {
    let mut db = DatabaseImpl::default();

    db.set_input_file("TextureA.jpg".to_owned(), "Texture A".to_owned());
    db.set_input_file("TextureB.jpg".to_owned(), "Texture B".to_owned());
    db.set_input_file("TextureEn.jpg".to_owned(), "Texture in English".to_owned());
    db.set_input_file("TextureFr.jpg".to_owned(), "Texture en Français".to_owned());
    db.set_input_file(
        "Atlas.entity".to_owned(),
        "TextureA.jpg,TextureB.jpg,TextureC.jpg".to_owned(),
    );

    db.set_platform(Platform::PS5);
    db.set_locale(Locale::French);
    db.set_target(Target::Client);

    db
}

fn main() {
    let db = setup();

    println!("Atlas {}", db.compile_atlas("Atlas.entity".to_owned()));
    println!("Material {}", db.compile_material());
}
