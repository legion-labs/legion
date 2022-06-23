use crate::compiler::CompilerStorage;

#[salsa::database(CompilerStorage)]
#[derive(Default)]
pub struct DatabaseImpl {
    storage: salsa::Storage<Self>,
}

/// This impl tells salsa where to find the salsa runtime.
impl salsa::Database for DatabaseImpl {}
