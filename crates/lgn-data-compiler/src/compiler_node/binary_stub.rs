use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};

use super::CompilerStub;
use crate::{
    compiler_api::{CompilationEnv, CompilationOutput, CompilerError, CompilerInfo},
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd, CompilerInfoCmdOutput},
    CompiledResource, CompilerHash,
};

pub(super) struct BinCompilerStub {
    pub(super) bin_path: PathBuf,
}

impl CompilerStub for BinCompilerStub {
    fn compiler_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<CompilerHash> {
        let cmd = CompilerHashCmd::new(env, Some(transform));
        let transforms = cmd
            .execute(&self.bin_path)
            .map(|output| output.compiler_hash_list)?;

        if transforms.len() == 1 && transforms[0].0 == transform {
            return Ok(transforms[0].1);
        }

        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unexpected CompilerHashCmd output",
        ))
    }

    fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        // does nothing as the compiler process is responsible for initialization.
        registry
    }

    fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        _registry: Arc<AssetRegistry>,
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
            .map_err(CompilerError::StdoutError)
    }

    fn info(&self) -> io::Result<Vec<CompilerInfo>> {
        let cmd = CompilerInfoCmd::default();
        cmd.execute(&self.bin_path).map(CompilerInfoCmdOutput::take)
    }
}
