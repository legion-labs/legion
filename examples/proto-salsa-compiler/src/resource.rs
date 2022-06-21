use proto_salsa_compiler::{BuildParams, CompilerError};

use crate::{atlas::AtlasCompiler, inputs::Inputs};

#[salsa::query_group(ResourceStorage)]
pub trait ResourceCompiler: Inputs + AtlasCompiler {
    fn compile_resource(
        &self,
        resource_path_id: String,
        build_params: BuildParams,
    ) -> Result<String, CompilerError>;
    fn add_runtime_dependencdy(&self, resource_path_id: String, build_params: BuildParams) -> i8;
}

pub fn compile_resource(
    db: &dyn ResourceCompiler,
    resource_path_id: String,
    build_params: BuildParams,
) -> Result<String, CompilerError> {
    let mut split_transforms: Vec<&str> = resource_path_id.split('(').collect();
    let source = db.input_file(split_transforms.last().unwrap().to_string());
    split_transforms.pop().unwrap();

    let mut transformed_value = source;

    for transform in split_transforms.into_iter().rev() {
        if transform == "compile_atlas" {
            transformed_value = db.compile_atlas(transformed_value, build_params.clone());
        } else {
            return Err(CompilerError::ParsingError);
        }
    }

    Ok(transformed_value)
}

pub fn add_runtime_dependencdy(
    db: &dyn ResourceCompiler,
    name: String,
    build_params: BuildParams,
) -> i8 {
    // Todo: Spawn a task to parallelize this build.
    db.compile_resource(name, build_params);
    // This return value is a firewall so the caller never gets invalidated on a runtime dependency.
    0
}
