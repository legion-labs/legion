use std::sync::Arc;

use crate::{BuildParams, CompilerError};

use crate::compiler::Compiler;

pub fn compile_entity(
    db: &dyn Compiler,
    resources_to_compile: String,
    build_params: Arc<BuildParams>,
) -> Vec<String> {
    // This compiler returns a vector of strings splitted by ;
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
