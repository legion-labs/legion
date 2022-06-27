use std::sync::Arc;

use crate::BuildParams;

use crate::compiler::Compiler;

pub fn compile_entity(
    db: &dyn Compiler,
    resources_to_compile: String,
    build_params: Arc<BuildParams>,
) -> Vec<String> {
    // This compiler executes the embedded expressions.
    let expressions: Vec<&str> = resources_to_compile.split(';').collect();

    let mut ret = Vec::new();
    for expression in expressions {
        ret.push(
            db.execute_expression(expression.to_string(), build_params.clone())
                .unwrap()
                .downcast_ref::<String>()
                .unwrap()
                .clone(),
        );
    }
    ret
}
