use std::sync::Arc;

use crate::{compiler::Compiler, texture::CompressionType, BuildParams};

pub fn compile_atlas(
    db: &dyn Compiler,
    atlas_expressions: String,
    build_params: Arc<BuildParams>,
) -> String {
    let mut atlas = String::new();
    let expressions: Vec<&str> = atlas_expressions.split(';').collect();

    for expression in expressions {
        let texture_path = db
            .execute_expression(expression.to_string(), build_params.clone())
            .downcast_ref::<String>()
            .unwrap()
            .clone();

        atlas.push_str((db.compile_texture(texture_path, CompressionType::BC4) + " + ").as_str());
    }

    atlas.clone()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::BuildParams;

    use crate::compiler::Compiler;
    use crate::tests::setup;

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
