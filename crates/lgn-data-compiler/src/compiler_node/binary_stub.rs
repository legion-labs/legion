use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use lgn_content_store::{indexing::SharedTreeIdentifier, Provider};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions, ResourcePathId, Transform};

use super::CompilerStub;
use crate::{
    compiler_api::{CompilationEnv, CompilationOutput, CompilerError, CompilerHash, CompilerInfo},
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd, CompilerInfoCmdOutput},
    CompiledResource,
};

pub(super) struct BinCompilerStub {
    pub(super) bin_path: PathBuf,
}

#[async_trait]
impl CompilerStub for BinCompilerStub {
    async fn compiler_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<CompilerHash> {
        let cmd = CompilerHashCmd::new(&self.bin_path, env, Some(transform));
        let transforms = cmd
            .execute()
            .await
            .map(|output| output.compiler_hash_list)?;

        if transforms.len() == 1 && transforms[0].0 == transform {
            return Ok(transforms[0].1);
        }

        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Unexpected CompilerHashCmd output",
        ))
    }

    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        // does nothing as the compiler process is responsible for initialization.
        registry
    }

    async fn compile(
        &self,
        compile_path: ResourcePathId,
        dependencies: &[ResourcePathId],
        derived_deps: &[CompiledResource],
        _registry: Arc<AssetRegistry>,
        _provider: &Provider,
        _runtime_manifest_id: &SharedTreeIdentifier,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        CompilerCompileCmd::new(
            &self.bin_path,
            &compile_path,
            dependencies,
            derived_deps,
            env,
        )
        .execute()
        .await
        .map(|output| CompilationOutput {
            compiled_resources: output.compiled_resources,
            resource_references: output.resource_references,
        })
        .map_err(CompilerError::StdoutError)
    }

    async fn info(&self) -> io::Result<Vec<CompilerInfo>> {
        CompilerInfoCmd::new(&self.bin_path)
            .execute()
            .await
            .map(CompilerInfoCmdOutput::take)
    }
}
