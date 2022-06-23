use std::sync::Arc;

use proto_salsa_compiler::{BuildParams, CompilerError};
//use rust_yard::shunting_yard::ShuntingYard;

use crate::{
    atlas::AtlasCompiler,
    collision::CollisionCompiler,
    inputs::Inputs,
    rust_yard::{token, ShuntingYard},
};

#[salsa::query_group(ResourceStorage)]
pub trait ResourceCompiler: Inputs + AtlasCompiler + CollisionCompiler {
    fn compile_resource(
        &self,
        resource_path_id: String,
        build_params: Arc<BuildParams>,
    ) -> Result<String, CompilerError>;

    fn add_runtime_dependency(
        &self,
        resource_path_id: String,
        build_params: Arc<BuildParams>,
    ) -> i8;
}

pub fn compile_resource(
    db: &dyn ResourceCompiler,
    expression: String,
    build_params: Arc<BuildParams>,
) -> Result<String, CompilerError> {
    execute_expression(expression.as_str(), build_params, db)
}

pub fn add_runtime_dependency(
    db: &dyn ResourceCompiler,
    resource_path_id: String,
    build_params: Arc<BuildParams>,
) -> i8 {
    // Todo: Spawn a task to parallelize this build.
    db.compile_resource(resource_path_id, build_params).unwrap();
    // This return value is a firewall so the caller never gets invalidated on a runtime dependency.
    0
}

pub fn execute_expression(
    expression: &str,
    build_params: Arc<BuildParams>,
    db: &dyn ResourceCompiler,
) -> Result<String, CompilerError> {
    let mut shunting_yard = ShuntingYard::new();

    let mut stack = shunting_yard.parse(expression).unwrap();
    println!("{}", shunting_yard.to_string());

    // Iterate over the tokens and calculate a result
    while stack.len() > 1 {
        let tok = stack.pop().unwrap();
        if let token::Token::Identifier(identifier) = tok {
            match &identifier as &str {
                "compile_atlas" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(arg_name)) = arg {
                        let textures_in_atlas = db.read(arg_name);
                        stack.push(token::Token::Identifier(
                            db.compile_atlas(textures_in_atlas, build_params.clone()),
                        ));
                    }
                }
                "compile_collision" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(arg_name)) = arg {
                        let collision_content = db.read(arg_name);
                        stack.push(token::Token::Identifier(
                            db.compile_collision(Arc::new(collision_content))
                                .to_string(),
                        ));
                    }
                }
                "read" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(arg_name)) = arg {
                        stack.push(token::Token::Identifier(db.read(arg_name)));
                    }
                }
                _ => {
                    println!("{}", shunting_yard.to_string());
                    return Err(CompilerError::ParsingError);
                }
            }
        }
    }
    let result = stack.pop();
    if let Some(token::Token::Identifier(result_identifier)) = result {
        Ok(result_identifier)
    } else {
        Err(CompilerError::ParsingError)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use proto_salsa_compiler::BuildParams;

    use crate::tests::setup;

    use super::execute_expression;

    #[test]
    fn simple_expression() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());
        let result = execute_expression("read(Atlas.entity)", build_params, &db).unwrap();
        assert_eq!(
            result,
            "meta(TextureA.meta);meta(TextureB.meta);meta(TextureC.meta)"
        );
    }
}
