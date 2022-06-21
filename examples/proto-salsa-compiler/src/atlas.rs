use proto_salsa_compiler::BuildParams;

use crate::inputs::Inputs;
use crate::meta::MetaCompiler;
use crate::texture::{CompressionType, TextureCompiler};

#[salsa::query_group(AtlasStorage)]
pub trait AtlasCompiler: Inputs + TextureCompiler + MetaCompiler {
    fn compile_atlas(&self, textures_in_atlas: String, build_params: BuildParams) -> String;
}

pub fn compile_atlas(
    db: &dyn AtlasCompiler,
    textures_in_atlas: String,
    build_params: BuildParams,
) -> String {
    let texture_metas: Vec<&str> = textures_in_atlas.split(',').collect();

    let mut atlas = String::new();
    for texture_meta in texture_metas {
        let path = db
            .meta_get_resource_path(texture_meta.to_owned(), build_params.clone())
            .unwrap();

        // In a proper build system, BC4 would also come from the meta.
        atlas.push_str((db.compile_texture(path, CompressionType::BC4) + " + ").as_str());
    }
    atlas.clone()
}
