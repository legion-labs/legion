use std::sync::Arc;

use proto_salsa_compiler::{BuildParams, CompilerError};
//use rust_yard::shunting_yard::ShuntingYard;

use crate::{
    atlas::AtlasCompiler,
    collision::CollisionCompiler,
    inputs::Inputs,
    meta::MetaCompiler,
    rust_yard::{token, ShuntingYard},
};

#[salsa::query_group(ResourceStorage)]
pub trait ResourceCompiler: Inputs + AtlasCompiler + CollisionCompiler + MetaCompiler {
    fn execute_expression(
        &self,
        resource_path_id: String,
        build_params: Arc<BuildParams>,
    ) -> Result<Vec<String>, CompilerError>;

    fn add_runtime_dependency(
        &self,
        resource_path_id: String,
        build_params: Arc<BuildParams>,
    ) -> i8;
}

pub fn execute_expression(
    db: &dyn ResourceCompiler,
    expression: String,
    build_params: Arc<BuildParams>,
) -> Result<Vec<String>, CompilerError> {
    let expressions: Vec<&str> = expression.split(';').collect();
    let mut result: Vec<String> = Vec::new();

    for expression in expressions {
        let mut shunting_yard = ShuntingYard::new();

        let mut stack = shunting_yard.parse(expression).unwrap();
        println!("{}", shunting_yard.to_string());

        // Iterate over the tokens and calculate a result
        while stack.len() > 1 {
            let tok = stack.pop().unwrap();
            if let token::Token::Identifier(identifier) = tok {
                match &identifier as &str {
                    "read" => {
                        let arg = stack.pop();

                        if let Some(token::Token::Identifier(file_name)) = arg {
                            stack.push(token::Token::Identifier(db.read(file_name)));
                        }
                    }
                    "compile_atlas" => {
                        let arg = stack.pop();

                        if let Some(token::Token::Identifier(content)) = arg {
                            stack.push(token::Token::Identifier(
                                db.compile_atlas(content, build_params.clone()),
                            ));
                        }
                    }
                    "compile_collision" => {
                        let arg = stack.pop();

                        if let Some(token::Token::Identifier(content)) = arg {
                            stack.push(token::Token::Identifier(
                                db.compile_collision(Arc::new(content)).to_string(),
                            ));
                        }
                    }
                    "meta" => {
                        let arg = stack.pop();

                        if let Some(token::Token::Identifier(content)) = arg {
                            stack.push(token::Token::Identifier(
                                db.meta_get_resource_path(content, build_params.clone())
                                    .unwrap(),
                            ));
                        }
                    }
                    _ => {
                        println!("{}", shunting_yard.to_string());
                        return Err(CompilerError::ParsingError);
                    }
                }
            }
        }
        let computed = stack.pop();
        if let Some(token::Token::Identifier(result_identifier)) = computed {
            result.push(result_identifier);
        } else {
            return Err(CompilerError::ParsingError);
        }
    }

    return Ok(result);
}

pub fn add_runtime_dependency(
    db: &dyn ResourceCompiler,
    resource_path_id: String,
    build_params: Arc<BuildParams>,
) -> i8 {
    // Todo: Spawn a task to parallelize this build.
    db.execute_expression(resource_path_id, build_params)
        .unwrap();
    // This return value is a firewall so the caller never gets invalidated on a runtime dependency.
    0
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
        let result =
            execute_expression(&db, "read(Atlas.entity)".to_string(), build_params).unwrap();
        assert_eq!(
            result[0],
            "meta(read(TextureA.meta));meta(read(TextureB.meta));meta(read(TextureC.meta))"
        );
    }
}
