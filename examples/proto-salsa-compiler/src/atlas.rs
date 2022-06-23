use std::sync::Arc;

use crate::BuildParams;

use crate::inputs::Inputs;
use crate::meta::MetaCompiler;
use crate::texture::{CompressionType, TextureCompiler};

#[salsa::query_group(AtlasStorage)]
pub trait AtlasCompiler: Inputs + TextureCompiler + MetaCompiler {
    fn compile_atlas(
        &self,
        textures_in_atlas: Vec<String>,
        build_params: Arc<BuildParams>,
    ) -> String;
}

pub fn compile_atlas(
    db: &dyn AtlasCompiler,
    atlas_textures_path: Vec<String>,
    build_params: Arc<BuildParams>,
) -> String {
    let mut atlas = String::new();
    for texture_path in atlas_textures_path {
        // In a proper build system, BC4 would also come from the meta.
        atlas.push_str(
            (db.compile_texture(texture_path.to_string(), CompressionType::BC4) + " + ").as_str(),
        );
    }
    atlas.clone()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::BuildParams;

    use crate::{atlas::AtlasCompiler, tests::setup};

    #[test]
    fn compile_atlas() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        assert_eq!(
            db.compile_atlas(
                vec!["TextureA.jpg".to_string(), "TextureB.png".to_string()],
                build_params
            ),
            "(Jpg Texture A compressed BC4) + (Png Texture B compressed BC4) + "
        );
    }
}
