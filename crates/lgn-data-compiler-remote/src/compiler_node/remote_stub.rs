use std::{
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use lgn_content_store::{indexing::SharedTreeIdentifier, Provider};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions, ResourcePathId, Transform};

// use super::remote_data_executor::collect_local_resources;
use lgn_data_compiler::{
    compiler_api::{CompilationEnv, CompilationOutput, CompilerError, CompilerHash, CompilerInfo},
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd, CompilerInfoCmdOutput},
    compiler_node::CompilerStub,
    CompiledResource,
};
pub struct RemoteCompilerStub {
    pub bin_path: PathBuf,
    pub server_addr: String,
}

#[async_trait]
impl CompilerStub for RemoteCompilerStub {
    async fn compiler_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<CompilerHash> {
        // Retrieving the hash is done locally for now.
        // FIXME: We should cache it in the CAS, and only run it once every time a compiler changes.

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
        data_provider: &Provider,
        _runtime_manifest_id: &SharedTreeIdentifier,
        env: &CompilationEnv,
    ) -> Result<CompilationOutput, CompilerError> {
        let _cmd = CompilerCompileCmd::new(
            self.bin_path.file_name().unwrap(),
            &compile_path,
            dependencies,
            derived_deps,
            env,
        );

        // let msg = collect_local_resources(
        //     &self.bin_path,
        //     resource_dir,
        //     &compile_path,
        //     dependencies,
        //     derived_deps,
        //     &cmd,
        //     data_provider,
        // )
        // .await?;
        let msg = "".to_owned();

        let result = crate::remote_service::client::send_receive_workload(&self.server_addr, msg);
        Ok(serde_json::from_str::<CompilationOutput>(&result)?)
    }

    async fn info(&self) -> io::Result<Vec<CompilerInfo>> {
        // Retrieving the info is done locally for now.
        // FIXME: We should cache it in the CAS, and only run it once every time a compiler changes.

        CompilerInfoCmd::new(&self.bin_path)
            .execute()
            .await
            .map(CompilerInfoCmdOutput::take)
    }
}
