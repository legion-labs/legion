use std::collections::VecDeque;
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

    let tokens = shunting_yard.parse(expression.as_str()).unwrap();
    println!("{}", shunting_yard.to_string());

    let mut stack: VecDeque<Arc<Box<dyn AnyEq>>> = VecDeque::new();

    // Iterate over the tokens and calculate a result
    for token in tokens {
        if let token::Token::Identifier(identifier) = token {
            let identifier_string = identifier.downcast_ref::<String>().unwrap();
            match identifier_string.as_str() {
                "read" => {
                    let file_name = stack
                        .pop_back()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    stack.push_front(Arc::new(Box::new(db.read(file_name))));
                }
                "atlas" => {
                    let content = stack
                        .pop_back()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    stack.push_front(Arc::new(Box::new(
                        db.compile_atlas(content, build_params.clone()),
                    )));
                }
                "query_collisions" => {
                    let content = stack
                        .pop_back()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    stack.push_front(Arc::new(Box::new(
                        db.query_collisions(content, build_params.clone()),
                    )));
                }
                "meta" => {
                    let content = stack
                        .pop_back()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    stack.push_front(Arc::new(Box::new(
                        db.meta_get_resource_path(content, build_params.clone()),
                    )));
                }
                "aabb" => {
                    let min_x = stack.pop_back().unwrap();
                    let min_y = stack.pop_back().unwrap();
                    let min_z = stack.pop_back().unwrap();
                    let max_x = stack.pop_back().unwrap();
                    let max_y = stack.pop_back().unwrap();
                    let max_z = stack.pop_back().unwrap();

                    stack.push_front(Arc::new(Box::new(db.compile_aabb(
                        Arc::new(min_x.downcast_ref::<String>().unwrap().clone()),
                        Arc::new(min_y.downcast_ref::<String>().unwrap().clone()),
                        Arc::new(min_z.downcast_ref::<String>().unwrap().clone()),
                        Arc::new(max_x.downcast_ref::<String>().unwrap().clone()),
                        Arc::new(max_y.downcast_ref::<String>().unwrap().clone()),
                        Arc::new(max_z.downcast_ref::<String>().unwrap().clone()),
                    ))));
                }
                "entity" => {
                    let content = stack
                        .pop_back()
                        .unwrap()
                        .downcast_ref::<String>()
                        .unwrap()
                        .clone();
                    stack.push_front(Arc::new(Box::new(
                        db.compile_entity(content, build_params.clone()),
                    )));
                }

                _ => {
                    // No function name match, we assume this is a function argument
                    stack.push_front(identifier.clone());
                }
            }
        }
    }

    let computed = stack.pop_back();
    if let Some(result_identifier) = computed {
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
            .execute_expression("read(Atlas.atlas)".to_string(), build_params)
            .unwrap();
        assert_eq!(
            result.downcast_ref::<String>().unwrap(),
            "meta(read(TextureA.meta));meta(read(TextureB.meta));meta(read(TextureC.meta))"
        );
    }

    #[test]
    fn composed_expression() {
        let db = setup();

        let build_params = Arc::new(BuildParams::default());

        let compiled_atlas = db
            .execute_expression("atlas(read(Atlas.atlas))".to_string(), build_params)
            .unwrap();

        assert_eq!(
            compiled_atlas.downcast_ref::<String>().unwrap(),
            "(Jpg Texture A compressed BC4) + (Png Texture B compressed BC4) + (Jpg Texture in English compressed BC4) + "
        );
    }
}
