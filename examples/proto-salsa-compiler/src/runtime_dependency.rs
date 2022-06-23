use std::sync::Arc;

use crate::BuildParams;
//use rust_yard::shunting_yard::ShuntingYard;

#[salsa::query_group(RuntimeDependencyStorage)]
pub trait RuntimeDependency {
    fn add_runtime_dependency(
        &self,
        resource_path_id: String,
        build_params: Arc<BuildParams>,
    ) -> i8;
}

pub fn add_runtime_dependency(
    db: &dyn RuntimeDependency,
    resource_path_id: String,
    build_params: Arc<BuildParams>,
) -> i8 {
    // Todo: Spawn a task to parallelize this build.
    //db.execute_expression(resource_path_id, build_params)
    //    .unwrap();
    // This return value is a firewall so the caller never gets invalidated on a runtime dependency.
    0
}
