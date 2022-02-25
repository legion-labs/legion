use std::{
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistry, AssetRegistryOptions};
use lgn_utils::find_monorepo_root;

use super::{remote_data_executor::collect_local_resources, CompilerStub};
use crate::{
    compiler_api::{CompilationEnv, CompilationOutput, CompilerError, CompilerInfo},
    compiler_cmd::{CompilerCompileCmd, CompilerHashCmd, CompilerInfoCmd, CompilerInfoCmdOutput},
    CompiledResource, CompilerHash,
};
pub struct RemoteCompilerStub {
    pub bin_path: PathBuf,
    pub server_addr: String,
}

impl CompilerStub for RemoteCompilerStub {
    fn compiler_hash(
        &self,
        transform: Transform,
        env: &CompilationEnv,
    ) -> io::Result<CompilerHash> {
        // Retrieving the hash is done locally for now.
        // FIXME: We should cache it in the CAS, and only run it once every time a compiler changes.

        let cmd = CompilerHashCmd::new(&self.bin_path, env, Some(transform));
        let transforms = cmd.execute().map(|output| output.compiler_hash_list)?;

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
        let workspace_root = find_monorepo_root()?;

        let mut cas_local_path = PathBuf::from_str(&workspace_root)?;
        cas_local_path = cas_local_path.join(&format!("{}", cas_addr));

        let cmd = CompilerCompileCmd::new(
            self.bin_path.file_name().unwrap(),
            &compile_path,
            dependencies,
            derived_deps,
            &ContentStoreAddr::from(cas_local_path.strip_prefix(cas_local_path.parent().unwrap())?), // only 'temp'
            resource_dir.strip_prefix(resource_dir.parent().unwrap())?, // only 'offline'
            env,
        );

        let archive = collect_local_resources(
            &self.bin_path,
            resource_dir,
            &cas_local_path,
            &compile_path,
            dependencies,
            derived_deps,
            serde_json::to_string_pretty(&cmd)?.as_str(),
        )?;

        let result = crate::remote_service::client::send_receive_workload("127.0.0.1", archive);

        let local_path = cas_local_path.parent().unwrap();

        // Write the archive to a file for debugging.
        /*let mut file_archive = PathBuf::from(local_path);
        file_archive.push(format!("{:#x}.zip", rand::random::<u64>()));
        fs::write(file_archive, &result)?;*/

        // Uncompress archive.
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&result)).unwrap();
        archive.extract(local_path).unwrap();

        // Return output.
        let mut output_file = PathBuf::from(local_path);
        output_file.push("output.json");
        let output_json = fs::read_to_string(&output_file)?;

        Ok(serde_json::from_str(&output_json)?)
    }

    fn info(&self) -> io::Result<Vec<CompilerInfo>> {
        // Retrieving the info is done locally for now.
        // FIXME: We should cache it in the CAS, and only run it once every time a compiler changes.

        CompilerInfoCmd::new(&self.bin_path)
            .execute()
            .map(CompilerInfoCmdOutput::take)
    }
}
