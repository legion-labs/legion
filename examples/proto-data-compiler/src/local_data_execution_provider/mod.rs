use std::{collections::VecDeque, sync::RwLock};

use async_trait::async_trait;
use dashmap::DashSet;
use service::{
    compiler_interface::{BuildParams, CompilationOutput, CompilerError},
    data_execution_provider::DataExecutionProvider,
    source_control::CommitRoot,
    ResourcePathId,
};
use waitmap::WaitMap;

// TODO : Support version_hash in resources to compile
pub struct LocalDataExecutionProvider {
    resources_to_compile: RwLock<VecDeque<(ResourcePathId, BuildParams, CommitRoot)>>,
    _resources_compiling: DashSet<ResourcePathId>,
    resources_compiled: WaitMap<ResourcePathId, CompilationOutput>,
}

#[async_trait]
impl DataExecutionProvider for LocalDataExecutionProvider {
    async fn compile(
        &self,
        id: ResourcePathId,
        build_params: BuildParams,
        commit_root: CommitRoot,
    ) -> Result<CompilationOutput, CompilerError> {
        self.resources_to_compile.write().unwrap().push_back((
            id.clone(),
            build_params,
            commit_root,
        ));

        // Wait for the worker to take the job and compile it
        match self.resources_compiled.wait(&id.clone()).await {
            Some(map_result) => {
                return Ok(map_result.value().clone());
            }
            None => Err(CompilerError::Cancelled),
        }
    }
    fn debug(&self) -> std::fmt::Result {
        Ok(())
    }

    async fn compilation_completed(
        &self,
        id: ResourcePathId,
        _build_params: BuildParams,
        _commit_root: CommitRoot,
        compilation_output: CompilationOutput,
    ) {
        self.resources_compiled.insert(id, compilation_output);
    }

    async fn poll_compilation_work(&self) -> Option<(ResourcePathId, BuildParams, CommitRoot)> {
        self.resources_to_compile.write().unwrap().pop_front()
    }
}

impl LocalDataExecutionProvider {
    pub fn new() -> Self {
        Self {
            resources_to_compile: RwLock::new(VecDeque::default()),
            _resources_compiling: DashSet::default(),
            resources_compiled: WaitMap::new(),
        }
    }
}
