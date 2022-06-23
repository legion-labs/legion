use strum_macros::{Display, EnumString};

use crate::inputs::Inputs;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumString, Display)]
pub enum CompressionType {
    BC1,
    BC2,
    BC3,
    BC4,
    BC5,
    BC6,
    BC7,
}

#[salsa::query_group(TextureStorage)]
pub trait TextureCompiler: Inputs {
    fn compile_texture(&self, name: String, compression: CompressionType) -> String;
    fn compile_jpg(&self, name: String, compression: CompressionType) -> String;
    fn compile_png(&self, name: String, compression: CompressionType) -> String;
}

pub fn compile_texture(
    db: &dyn TextureCompiler,
    name: String,
    compression: CompressionType,
) -> String {
    println!("compile_texture {}", name);
    let filename_split: Vec<&str> = name.split('.').collect();
    let extension = filename_split[1];
    if extension == "jpg" {
        db.compile_jpg(name, compression)
    } else if extension == "png" {
        db.compile_png(name, compression)
    } else {
        "Could not compiled texture".to_string()
    }
}

pub fn compile_jpg(db: &dyn TextureCompiler, name: String, compression: CompressionType) -> String {
    let mut result = "(Jpg ".to_owned();
    result.push_str(db.read(name).as_str());
    result.push_str(format!(" compressed {})", compression).as_str());
    result
}

pub fn compile_png(db: &dyn TextureCompiler, name: String, compression: CompressionType) -> String {
    let mut result = "(Png ".to_owned();
    result.push_str(db.read(name).as_str());
    result.push_str(format!(" compressed {})", compression).as_str());
    result
}
