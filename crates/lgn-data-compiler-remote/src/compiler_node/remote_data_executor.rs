use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use lgn_content_store::{Config, Identifier, Provider};
use lgn_data_compiler::{compiler_api::CompilerError, compiler_cmd::CompilerCompileCmd};
use lgn_tracing::info;
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};

use crate::node_crunch::nc_error::NCError;

/// The outgoing message for the Data-Executor server, requesting a compilation.
#[derive(Serialize, Deserialize)]
pub struct CompileMessage {
    pub build_script: CompilerCompileCmd,
    pub files_to_package: Vec<(String, Identifier)>,
}

#[allow(dead_code)]
async fn deploy_remotely(
    provider: &Provider,
    full_file_path: &Path,
    strip_prefix: &Path,
    files_to_package: Arc<RwLock<Vec<(String, Identifier)>>>,
) -> Result<(), CompilerError> {
    let relative_path = full_file_path.strip_prefix(strip_prefix).unwrap();
    let buf = tokio::fs::read(&full_file_path).await?;
    let content_hash = provider.write(&buf).await?;
    files_to_package
        .write()
        .unwrap()
        .push((relative_path.to_string_lossy().to_string(), content_hash));
    Ok(())
}

#[allow(dead_code)]
/// Upload the data compiler & its associated input dependencies to the CAS and create a message.
pub(crate) async fn collect_local_resources(
    executable: &Path,
    build_script: &CompilerCompileCmd,
    data_provider: &Provider,
) -> Result<String, CompilerError> {
    let files_to_package: Arc<RwLock<Vec<(String, Identifier)>>> =
        Arc::new(RwLock::new(Vec::new()));

    // Write the compiler .exe
    deploy_remotely(
        data_provider,
        executable,
        executable.parent().unwrap(),
        files_to_package.clone(),
    )
    .await?;

    // Write the build script into the message.
    let msg = CompileMessage {
        build_script: (*build_script).clone(),
        files_to_package: files_to_package.read().unwrap().clone(),
    };
    Ok(serde_json::to_string_pretty(&msg)?)
}

pub(crate) async fn deploy_files(
    provider: &Provider,
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

        let data = provider.read(&file.1).await?;

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
    let output = msg.build_script.execute_with_cwd(&out_folder).await?;

    // Compress the outcome
    Ok(serde_json::to_string_pretty(&output)?)
}
