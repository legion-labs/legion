use std::{io, path::Path, sync::Arc};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};

use super::CompilerStub;
use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, CompilerDescriptor, CompilerError, CompilerInfo,
    },
    CompiledResource, CompilerHash,
};

pub(super) struct InProcessCompilerStub {
    pub(super) descriptor: &'static CompilerDescriptor,
}

impl CompilerStub for InProcessCompilerStub {
    fn compiler_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<CompilerHash> {
        if transform != *self.descriptor.transform {
            return Err(io::Error::new(io::ErrorKind::Other, "Transform mismatch"));
        }
        let hash = self.descriptor.compiler_hash(env);
        Ok(hash)
    }

    fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        (self.descriptor.init_func)(registry)
    }

    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        registry: Arc<AssetRegistry>,
        cas_addr: ContentStoreAddr,
        _project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        self.descriptor.compile(
            compile_path,
            dependencies,
            derived_deps,
            registry,
            cas_addr,
            env,
        )
    }

    fn info(&self) -> io::Result<Vec<CompilerInfo>> {
        let info = CompilerInfo {
            build_version: self.descriptor.build_version.to_string(),
            code_version: self.descriptor.code_version.to_string(),
            data_version: self.descriptor.data_version.to_string(),
            transform: *self.descriptor.transform,
        };
        Ok(vec![info])
    }
}
