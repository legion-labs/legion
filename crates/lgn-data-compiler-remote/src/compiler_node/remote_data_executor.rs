use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::node_crunch::nc_error::NCError;
use lgn_data_offline::ResourcePathId;
use serde::{Deserialize, Serialize};

use lgn_content_store::{Config, ContentProvider, ContentReaderExt, ContentWriterExt, Identifier};
use lgn_data_compiler::{
    compiler_api::CompilerError, compiler_cmd::CompilerCompileCmd, CompiledResource,
};
use lgn_tracing::info;
use tokio::{fs, io::AsyncWriteExt};

/// The outgoing message for the Data-Executor server, requesting a compilation.
#[derive(Serialize, Deserialize)]
pub struct CompileMessage {
    pub build_script: CompilerCompileCmd,
    pub files_to_package: Vec<(String, Identifier)>,
}

async fn deploy_remotely(
    provider: &(dyn ContentProvider + Send + Sync),
    full_file_path: &Path,
    strip_prefix: &Path,
    files_to_package: Arc<RwLock<Vec<(String, Identifier)>>>,
) -> Result<(), CompilerError> {
    let relative_path = full_file_path.strip_prefix(strip_prefix).unwrap();
    let buf = tokio::fs::read(&full_file_path).await?;
    let content_hash = provider.write_content(&buf).await?;
    files_to_package
        .write()
        .unwrap()
        .push((relative_path.to_string_lossy().to_string(), content_hash));
    Ok(())
}

async fn write_res(
    provider: &(dyn ContentProvider + Send + Sync),
    res: &ResourcePathId,
    resource_dir: &Path,
    files_to_package: Arc<RwLock<Vec<(String, Identifier)>>>,
) -> Result<(), CompilerError> {
    let mut source = PathBuf::from(resource_dir);
    source.push(&res.source_resource().id.resource_path());

    deploy_remotely(
        provider,
        &source,
        PathBuf::from(resource_dir).parent().unwrap(),
        files_to_package,
    )
    .await
}

/// Upload the data compiler & its associated input dependencies to the CAS and create a message.
pub(crate) async fn collect_local_resources(
    executable: &Path,
    resource_dir: &Path,
    compile_path: &ResourcePathId,
    dependencies: &[ResourcePathId],
    _derived_deps: &[CompiledResource],
    build_script: &CompilerCompileCmd,
    data_content_provider: impl ContentProvider + Send + Sync,
) -> Result<String, CompilerError> {
    let files_to_package: Arc<RwLock<Vec<(String, Identifier)>>> =
        Arc::new(RwLock::new(Vec::new()));

    // Write the compiler .exe
    deploy_remotely(
        &data_content_provider,
        executable,
        executable.parent().unwrap(),
        files_to_package.clone(),
    )
    .await?;

    // Write the main resource.
    write_res(
        &data_content_provider,
        compile_path,
        resource_dir,
        files_to_package.clone(),
    )
    .await?;

    // Write the direct offline dependencies
    for dep in dependencies {
        write_res(
            &data_content_provider,
            dep,
            resource_dir,
            files_to_package.clone(),
        )
        .await?;
    }

    // Write the build script into the message.
    let msg = CompileMessage {
        build_script: (*build_script).clone(),
        files_to_package: files_to_package.read().unwrap().clone(),
    };
    Ok(serde_json::to_string_pretty(&msg)?)
}

pub(crate) async fn deploy_files(
    provider: &(dyn ContentProvider + Send + Sync),
    files: &[(String, Identifier)],
    out_folder: &Path,
) -> Result<(), CompilerError> {
    info!("deploying {} files.", files.len());
    for file in files {
        let mut file_name = PathBuf::new();
        file_name.push(out_folder);
        file_name.push(&file.0);

        info!("deploying '{:?}'", file_name);

        fs::create_dir_all(file_name.parent().unwrap()).await?;

        let data = provider.read_content(&file.1).await?;

        let mut output = {
            #[cfg(unix)]
            let is_exec = file_name
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("compiler-");

            let mut options = tokio::fs::OpenOptions::new();
            options.write(true);
            #[cfg(unix)]
            if is_exec {
                options.mode(755);
            }
            options.create(true);
            options.open(file_name).await?
        };

        output.write_all(&data).await?;
    }
    Ok(())
}

/// Given a message, retrieve the inputs & data a compiler from the CAS, and execute the data compiler remotely.
pub(crate) async fn execute_sandbox_compiler(input_msg: &str) -> Result<String, NCError> {
    let out_folder = tempfile::tempdir()?;
    let msg: CompileMessage = serde_json::from_str(input_msg)?;

    let content_provider = Config::load_and_instantiate_volatile_provider()
        .await
        .map_err(|err| {
            CompilerError::RemoteExecution(format!("failed to create content provider: {}", err))
        })?;

    // Retrieve all inputs from the CAS.
    deploy_files(&content_provider, &msg.files_to_package, out_folder.path()).await?;

    // Run
    let output = msg.build_script.execute_with_cwd(&out_folder)?;

    // Compress the outcome
    Ok(serde_json::to_string_pretty(&output)?)
}
