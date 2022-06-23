use std::sync::Arc;

use crate::{compiler::Compiler, BuildParams, CompilerError};

// Only supporting locale for now, but it would be the same for platform & target specifiers
pub fn meta_get_resource_path(
    db: &dyn Compiler,
    meta_content: String,
    build_params: Arc<BuildParams>,
) -> Result<String, CompilerError> {
    let split_meta: Vec<&str> = meta_content.split('\n').collect();

    for meta in split_meta {
        let split_lang_filepath: Vec<&str> = meta.split(':').collect();
        let locale_meta = split_lang_filepath[0];
        if locale_meta == "Default" || locale_meta == build_params.locale.to_string() {
            return Ok(split_lang_filepath[1].to_string());
        }
    }

    Err(CompilerError::ParsingError)
}
