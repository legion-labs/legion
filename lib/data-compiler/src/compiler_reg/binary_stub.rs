use std::{
    io,
    path::{Path, PathBuf},
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::ResourcePathId;

use super::CompilerStub;
use crate::{
    compiler_api::{CompilationEnv, CompilationOutput, CompilerError, CompilerInfo},
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd},
    CompiledResource, CompilerHash,
};

pub(super) struct BinCompilerStub {
    pub(super) bin_path: PathBuf,
}

impl CompilerStub for BinCompilerStub {
    fn compiler_hash(&self, env: &CompilationEnv) -> io::Result<CompilerHash> {
        let cmd = CompilerHashCmd::new(env);
        cmd.execute(&self.bin_path)
            .map(|output| output.compiler_hash)
    }

    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        cas_addr: ContentStoreAddr,
        resource_dir: &Path,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        let mut cmd = CompilerCompileCmd::new(
            &compile_path,
            dependencies,
            derived_deps,
            &cas_addr,
            resource_dir,
            env,
        );

        cmd.execute(&self.bin_path)
            .map(|output| CompilationOutput {
                compiled_resources: output.compiled_resources,
                resource_references: output.resource_references,
            })
            .map_err(|_e| CompilerError::StdoutError)
    }

    fn info(&self) -> io::Result<CompilerInfo> {
        let cmd = CompilerInfoCmd::default();
        cmd.execute(&self.bin_path).map(|output| CompilerInfo {
            build_version: output.build_version,
            code_version: output.code_version,
            data_version: output.data_version,
            transform: output.transform,
        })
    }
}
