use std::sync::Arc;

use proto_salsa_compiler::BuildParams;

use crate::inputs::Inputs;
use crate::meta::MetaCompiler;
use crate::texture::{CompressionType, TextureCompiler};

#[salsa::query_group(AtlasStorage)]
pub trait AtlasCompiler: Inputs + TextureCompiler + MetaCompiler {
    fn compile_atlas(&self, textures_in_atlas: String, build_params: Arc<BuildParams>) -> String;
}

pub fn compile_atlas(
    db: &dyn AtlasCompiler,
    atlas_textures_path: String,
    build_params: Arc<BuildParams>,
) -> String {
    let atlas_textures: Vec<&str> = atlas_textures_path.split(';').collect();

    let mut atlas = String::new();
    for texture in atlas_textures {
        // In a proper build system, BC4 would also come from the meta.
        atlas.push_str(
            (db.compile_texture(texture.to_string(), CompressionType::BC4) + " + ").as_str(),
        );
    }
    atlas.clone()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use proto_salsa_compiler::BuildParams;

    use crate::{atlas::AtlasCompiler, tests::setup};

    #[test]
    fn compile_atlas() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        assert_eq!(
            db.compile_atlas("TextureA.jpg;TextureB.png".to_string(), build_params),
            "(Jpg Texture A compressed BC4) + (Png Texture B compressed BC4) + "
        );
    }
}
