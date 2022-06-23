use std::sync::Arc;

use crate::compiler::Compiler;
use crate::rust_yard::token;
use crate::rust_yard::ShuntingYard;
use crate::BuildParams;
use crate::CompilerError;

pub fn execute_expression(
    db: &dyn Compiler,
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
                                db.compile_atlas(vec![content], build_params.clone()),
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
                    "entity" => {
                        let arg = stack.pop();

                        if let Some(token::Token::Identifier(content)) = arg {
                            stack.push(token::Token::Identifier(
                                db.compile_entity(content, build_params.clone()),
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

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::compiler::Compiler;
    use crate::tests::setup;
    use crate::BuildParams;

    #[test]
    fn simple_expression() {
        let db = setup();
        let build_params = Arc::new(BuildParams::default());

        let result = db
            .execute_expression("read(Atlas.entity)".to_string(), build_params)
            .unwrap();
        assert_eq!(
            result[0],
            "meta(read(TextureA.meta));meta(read(TextureB.meta));meta(read(TextureC.meta))"
        );
    }
}
