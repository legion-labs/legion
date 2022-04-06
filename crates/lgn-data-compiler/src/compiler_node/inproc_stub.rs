use async_trait::async_trait;
use lgn_content_store::ContentProvider;
use std::{io, path::Path, sync::Arc};

use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};

use super::CompilerStub;
use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerDescriptor, CompilerError,
        CompilerHash, CompilerInfo,
    },
    CompiledResource,
};

pub(super) struct InProcessCompilerStub {
    pub(super) descriptor: &'static CompilerDescriptor,
    pub(super) compiler: Box<dyn Compiler + Send + Sync>,
}
impl InProcessCompilerStub {
    pub(super) fn new(descriptor: &'static CompilerDescriptor) -> Self {
        Self {
            descriptor,
            compiler: descriptor.instantiate_compiler(),
        }
    }
}

#[async_trait]
impl CompilerStub for InProcessCompilerStub {
    async fn compiler_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<CompilerHash> {
        if transform != *self.descriptor.transform {
            return Err(io::Error::new(io::ErrorKind::Other, "Transform mismatch"));
        }
        let hash = self
            .descriptor
            .compiler_hash(self.compiler.as_ref(), env)
            .await;
        Ok(hash)
    }

    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        self.compiler.init(registry).await
    }

    async fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        registry: Arc<AssetRegistry>,
        data_content_provider: &(dyn ContentProvider + Send + Sync),
        _project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        self.descriptor
            .compile(
                self.compiler.as_ref(),
                compile_path,
                dependencies,
                derived_deps,
                registry,
                data_content_provider,
                env,
            )
            .await
    }

    async fn info(&self) -> io::Result<Vec<CompilerInfo>> {
        let info = CompilerInfo {
            build_version: self.descriptor.build_version.to_string(),
            code_version: self.descriptor.code_version.to_string(),
            data_version: self.descriptor.data_version.to_string(),
            transform: *self.descriptor.transform,
        };
        Ok(vec![info])
    }
}
