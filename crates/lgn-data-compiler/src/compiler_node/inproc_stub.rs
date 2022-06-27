use std::{io, sync::Arc};

use async_trait::async_trait;
use lgn_content_store::{indexing::SharedTreeIdentifier, Provider};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions, ResourcePathId, Transform};

use super::CompilerStub;
use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerDescriptor, CompilerError,
        CompilerHash, CompilerInfo,
    },
    CompiledResource, CompiledResources,
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
        if transform != *self.descriptor.transform
            && !self.descriptor.transform.is_wildcard_for(&transform)
        {
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
        volatile_provider: Arc<Provider>,
        persistent_provider: Arc<Provider>,
        _source_manifest_id: &SharedTreeIdentifier,
        runtime_manifest_id: &SharedTreeIdentifier,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        // build temporary runtime manifest
        let previous_manifest_id = runtime_manifest_id.read();
        let manifest = CompiledResources {
            compiled_resources: derived_deps.to_vec(),
        };
        let manifest_id = manifest
            .into_rt_manifest(Arc::clone(&volatile_provider), |_rpid| true)
            .await;
        runtime_manifest_id.write(manifest_id);

        let result = self
            .descriptor
            .compile(
                self.compiler.as_ref(),
                compile_path,
                dependencies,
                derived_deps,
                registry,
                Arc::clone(&volatile_provider),
                Arc::clone(&persistent_provider),
                env,
            )
            .await;

        runtime_manifest_id.write(previous_manifest_id);

        result
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
