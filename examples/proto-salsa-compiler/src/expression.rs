use std::any;
use std::any::Any;
use std::any::TypeId;
use std::sync::Arc;

use crate::compiler::AnyEq;
use crate::compiler::Compiler;
use crate::rust_yard::token;
use crate::rust_yard::ShuntingYard;
use crate::BuildParams;
use crate::CompilerError;

pub fn execute_expression(
    db: &dyn Compiler,
    expression: String,
    build_params: Arc<BuildParams>,
) -> Result<Arc<Box<dyn AnyEq>>, CompilerError> {
    let mut shunting_yard = ShuntingYard::new();

    let mut stack = shunting_yard.parse(expression.as_str()).unwrap();
    println!("{}", shunting_yard.to_string());

    // Iterate over the tokens and calculate a result
    while stack.len() > 1 {
        let tok = stack.pop().unwrap();
        if let token::Token::Identifier(identifier) = tok {
            match identifier.downcast_ref::<String>().unwrap().as_str() {
                "read" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(file_name_identifier)) = arg {
                        let file_name = file_name_identifier
                            .downcast_ref::<String>()
                            .unwrap()
                            .clone();
                        stack.push(token::Token::Identifier(Arc::new(Box::new(
                            db.read(file_name),
                        ))));
                    }
                }
                "atlas" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(content_identifier)) = arg {
                        let content = content_identifier
                            .downcast_ref::<Vec<String>>()
                            .unwrap()
                            .clone();
                        stack.push(token::Token::Identifier(Arc::new(Box::new(
                            db.compile_atlas(content, build_params.clone()),
                        ))));
                    }
                }
                "collision" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(content)) = arg {
                        stack.push(token::Token::Identifier(Arc::new(Box::new(
                            db.compile_collision(Arc::new(
                                content.downcast_ref::<String>().unwrap().clone(),
                            ))
                            .to_string(),
                        ))));
                    }
                }
                "meta" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(content)) = arg {
                        stack.push(token::Token::Identifier(Arc::new(Box::new(
                            db.meta_get_resource_path(
                                content.downcast_ref::<String>().unwrap().clone(),
                                build_params.clone(),
                            )
                            .unwrap(),
                        ))));
                    }
                }
                "entity" => {
                    let arg = stack.pop();

                    if let Some(token::Token::Identifier(content)) = arg {
                        stack.push(token::Token::Identifier(Arc::new(Box::new(
                            db.compile_entity(
                                content.downcast_ref::<String>().unwrap().clone(),
                                build_params.clone(),
                            ),
                        ))));
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
        Ok(result_identifier)
    } else {
        Err(CompilerError::ParsingError)
    }
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
            result.downcast_ref::<String>().unwrap(),
            "meta(read(TextureA.meta));meta(read(TextureB.meta));meta(read(TextureC.meta))"
        );
    }
}
