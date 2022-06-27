use std::sync::Arc;

use crate::BuildParams;

use crate::compiler::Compiler;

pub fn compile_entity(
    _db: &dyn Compiler,
    resources_to_compile: String,
    _build_params: Arc<BuildParams>,
) -> Vec<String> {
    // This compiler returns a vector of strings splitted by ;
    resources_to_compile
        .split(';')
        .map(|val| val.to_string())
        .collect()
}
