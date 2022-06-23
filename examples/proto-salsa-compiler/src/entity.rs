use std::sync::Arc;

use crate::BuildParams;

use crate::{inputs::Inputs, runtime_dependency::RuntimeDependency};

#[salsa::query_group(EntityStorage)]
pub trait EntityCompiler: Inputs + RuntimeDependency {
    fn compile_entity(&self, name: String, build_params: Arc<BuildParams>) -> String;
}

pub fn compile_entity(
    db: &dyn EntityCompiler,
    resources_to_compile: String,
    build_params: Arc<BuildParams>,
) -> String {
    let split_resources: Vec<&str> = resources_to_compile.split(';').collect();

    // Here we would send back the jobs to the scheduler.
    for resource in split_resources {
        db.add_runtime_dependency(resource.to_string(), build_params.clone());
    }

    // This compiler is a passthrough for us
    resources_to_compile
}
