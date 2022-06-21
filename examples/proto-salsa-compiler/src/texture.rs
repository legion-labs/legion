use crate::inputs::Inputs;

#[salsa::query_group(TextureStorage)]
pub trait TextureCompiler: Inputs {
    fn compile_texture(&self, name: String) -> String;
}

pub fn compile_texture(db: &dyn TextureCompiler, name: String) -> String {
    let mut result = "Compiled ".to_owned();
    result.push_str(db.input_file(name).as_str());
    result
}
