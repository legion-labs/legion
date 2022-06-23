use atlas::AtlasStorage;
use entity::EntityStorage;
use material::MaterialStorage;

use crate::inputs::InputsStorage;
use crate::texture::TextureStorage;
use crate::{
    collision::CollisionStorage, expression::ResourceStorage, meta::MetaStorage,
    navmesh::NavmeshStorage, package::PackageStorage,
};

mod atlas;
mod collision;
mod entity;
mod expression;
mod inputs;
mod material;
mod meta;
mod navmesh;
mod package;
mod rust_yard;
mod texture;

#[salsa::database(
    InputsStorage,
    AtlasStorage,
    MaterialStorage,
    TextureStorage,
    PackageStorage,
    EntityStorage,
    MetaStorage,
    ResourceStorage,
    NavmeshStorage,
    CollisionStorage
)]
#[derive(Default)]
pub struct DatabaseImpl {
    storage: salsa::Storage<Self>,
}

/// This impl tells salsa where to find the salsa runtime.
impl salsa::Database for DatabaseImpl {}

fn main() {}

#[cfg(test)]
mod tests {
    use proto_salsa_compiler::{BuildParams, Locale, Platform, Target};

    use crate::collision::AABBCollision;
    use crate::inputs::Inputs;
    use crate::navmesh::NavmeshCompiler;
    use crate::DatabaseImpl;
    use crate::{atlas::AtlasCompiler, package::PackageCompiler};

    pub fn setup() -> DatabaseImpl {
        let mut db = DatabaseImpl::default();

        db.set_read(
            "TextureA.meta".to_string(),
            "Default:TextureA.jpg".to_string(),
        );
        db.set_read("TextureA.jpg".to_string(), "Texture A".to_string());

        db.set_read(
            "TextureB.meta".to_string(),
            "Default:TextureB.png".to_string(),
        );
        db.set_read("TextureB.png".to_string(), "Texture B".to_string());

        db.set_read(
            "TextureC.meta".to_string(),
            "English:TextureCEn.jpg\nFrench:TextureCFr.png".to_string(),
        );
        db.set_read(
            "TextureCEn.jpg".to_string(),
            "Texture in English".to_string(),
        );
        db.set_read(
            "TextureCFr.png".to_string(),
            "Texture en Français".to_string(),
        );

        db.set_read(
            "Atlas.entity".to_string(),
            "TextureA.meta,TextureB.meta,TextureC.meta".to_string(),
        );

        db.set_read(
            "MyWorld.entity".to_string(),
            r#"compile_atlas(Atlas.entity);compile_collision(Car.entity);compile_collision(Tree.entity)"#
                .to_string(),
        );

        db.set_read("Car.entity".to_string(), "5,5,5,10,10,10".to_string());
        db.set_read("Tree.entity".to_string(), "30,30,30,50,60,70".to_string());

        db
    }

    #[test]
    fn compile_all() {
        let db = setup();

        db.package_see_ps5();
    }

    #[test]
    fn incremental_compilation() {
        let db = setup();

        let build_params = BuildParams::new(Platform::PS5, Target::Client, Locale::English);
        let atlas_content = db.read("Atlas.entity".to_string());
        println!("Atlas: {}", db.compile_atlas(atlas_content, build_params));

        db.package_see_ps5();
    }

    #[test]
    fn navmesh_add_object() {
        let db = setup();

        db.compile_navmesh(AABBCollision {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 10,
            max_y: 10,
            max_z: 10,
        });
    }

    #[test]
    fn navmesh_remove_object() {
        let db = setup();

        db.compile_navmesh(AABBCollision {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 10,
            max_y: 10,
            max_z: 10,
        });
    }

    #[test]
    fn navmesh_move_object() {}
}
