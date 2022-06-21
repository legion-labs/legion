use proto_salsa_compiler::BuildParams;

use crate::{inputs::Inputs, resource::ResourceCompiler};

#[salsa::query_group(EntityStorage)]
pub trait EntityCompiler: Inputs + ResourceCompiler {
    fn compile_entity(&self, name: String, build_params: BuildParams) -> String;
}

pub fn compile_entity(db: &dyn EntityCompiler, name: String, build_params: BuildParams) -> String {
    let resources_to_compile = db.input_file(name);

    let split_resources: Vec<&str> = resources_to_compile.split(',').collect();
    for resource in split_resources {
        db.add_runtime_dependencdy(resource.to_string(), build_params.clone());
    }

    // This compiler is a passthrough for us
    resources_to_compile
}
