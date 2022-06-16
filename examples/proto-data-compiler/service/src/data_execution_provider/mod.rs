use async_trait::async_trait;

use crate::{
    compiler_interface::{BuildParams, CompilationOutput, CompilerError},
    source_control::CommitRoot,
    ResourcePathId,
};

#[async_trait]
pub trait DataExecutionProvider: Send + Sync {
    async fn compile(
        &self,
        id: ResourcePathId,
        build_params: BuildParams,
        commit_root: CommitRoot,
    ) -> Result<CompilationOutput, CompilerError>;

    fn debug(&self) -> std::fmt::Result;

    // Need to discuss with Kris : I'm not sure I like this function,
    // but I don't know blocker are blocked without this function
    // since it has to wait after completion of all runtime_references
    // ... Maybe a queue inside the worker to know what we are waiting after ... ?
    // Sounds ugly.
    async fn compilation_completed(
        &self,
        id: ResourcePathId,
        build_params: BuildParams,
        commit_root: CommitRoot,
        compilation_output: CompilationOutput,
    );

    async fn poll_compilation_work(&self) -> Option<(ResourcePathId, BuildParams, CommitRoot)>;
}

impl std::fmt::Debug for dyn DataExecutionProvider {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug()
    }
}
