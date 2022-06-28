use strum_macros::{Display, EnumString};

use crate::compiler::Compiler;

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

pub fn compile_texture(db: &dyn Compiler, name: String, compression: CompressionType) -> String {
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

pub fn compile_jpg(db: &dyn Compiler, name: String, compression: CompressionType) -> String {
    let mut result = "(Jpg ".to_owned();
    result.push_str(db.read(name).as_str());
    result.push_str(format!(" compressed {})", compression).as_str());
    result
}

pub fn compile_png(db: &dyn Compiler, name: String, compression: CompressionType) -> String {
    let mut result = "(Png ".to_owned();
    result.push_str(db.read(name).as_str());
    result.push_str(format!(" compressed {})", compression).as_str());
    result
}

#[cfg(test)]
mod tests {
    use crate::{compiler::Compiler, tests::setup, texture::CompressionType};

    #[test]
    fn compile_jpg() {
        let db = setup();

        assert_eq!(
            db.compile_texture("TextureA.jpg".to_string(), CompressionType::BC7),
            "(Jpg Texture A compressed BC7)"
        );
    }

    #[test]
    fn compile_png() {
        let db = setup();

        assert_eq!(
            db.compile_texture("TextureB.png".to_string(), CompressionType::BC3),
            "(Png Texture B compressed BC3)"
        );
    }
}
