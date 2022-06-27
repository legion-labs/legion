use std::sync::Arc;

use crate::{compiler::Compiler, BuildParams};

pub fn add_runtime_dependency(
    db: &dyn Compiler,
    resource_path_id: String,
    build_params: Arc<BuildParams>,
) -> i8 {
    // Todo: Spawn a task to parallelize this build.
    db.execute_expression(resource_path_id, build_params);
    // This return value is a firewall so the caller never gets invalidated on a runtime dependency.
    0
}
