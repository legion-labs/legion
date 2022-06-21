use proto_salsa_compiler::{BuildParams, CompilerError};

use crate::inputs::Inputs;

#[salsa::query_group(MetaStorage)]
pub trait MetaCompiler: Inputs {
    fn meta_get_resource_path(
        &self,
        meta_content: String,
        build_params: BuildParams,
    ) -> Result<String, CompilerError>;
}

// Only supporting locale for now, but it would be the same for platform & target specifiers
pub fn meta_get_resource_path(
    db: &dyn MetaCompiler,
    meta_file: String,
    build_params: BuildParams,
) -> Result<String, CompilerError> {
    let meta_content = db.input_file(meta_file.to_string());
    let split_meta: Vec<&str> = meta_content.split('\n').collect();

    println!("{}", build_params.locale);
    for meta in split_meta {
        let split_lang_filepath: Vec<&str> = meta.split(':').collect();
        let locale_meta = split_lang_filepath[0];
        if locale_meta == "Default" || locale_meta == build_params.locale.to_string() {
            return Ok(split_lang_filepath[1].to_string());
        }
    }

    Err(CompilerError::ParsingError)
}
