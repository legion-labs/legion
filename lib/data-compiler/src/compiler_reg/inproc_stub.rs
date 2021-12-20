use std::{io, path::Path};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::ResourcePathId;

use crate::{
    compiler_api::{
        CompilationEnv, CompilationOutput, CompilerDescriptor, CompilerError, CompilerInfo,
    },
    CompiledResource, CompilerHash,
};

use super::CompilerStub;

pub(super) struct InProcessCompilerStub {
    pub(super) descriptor: &'static CompilerDescriptor,
}

impl CompilerStub for InProcessCompilerStub {
    fn compiler_hash(&self, env: &CompilationEnv) -> io::Result<CompilerHash> {
        let hash = (self.descriptor.compiler_hash_func)(
            self.descriptor.code_version,
            self.descriptor.data_version,
            env,
        );
        Ok(hash)
    }

    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        project_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        self.descriptor.compile(
            compile_path,
            dependencies,
            derived_deps,
            cas_addr,
            project_dir,
            env,
        )
    }

    fn info(&self) -> io::Result<CompilerInfo> {
        let info = CompilerInfo {
            build_version: self.descriptor.build_version.to_string(),
            code_version: self.descriptor.code_version.to_string(),
            data_version: self.descriptor.data_version.to_string(),
            transform: *self.descriptor.transform,
        };
        Ok(info)
    }
}
