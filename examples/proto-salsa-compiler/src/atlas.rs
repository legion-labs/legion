use crate::inputs::Inputs;
use crate::texture::TextureCompiler;

#[salsa::query_group(AtlasStorage)]
pub trait AtlasCompiler: Inputs + TextureCompiler {
    fn compile_atlas(&self, name: String) -> String;
}

pub fn compile_atlas(db: &dyn AtlasCompiler, name: String) -> String {
    let file_content: String = db.input_file(name);
    let texture_paths = file_content.split(",");

    let mut atlas = String::new();
    for texture_path in texture_paths {
        atlas.push_str((db.compile_texture(texture_path.to_owned()) + " + ").as_str());
    }
    atlas.to_owned()
}
