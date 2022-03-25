use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use crate::node_crunch::nc_error::NCError;
use lgn_data_offline::ResourcePathId;
use serde::{Deserialize, Serialize};

use lgn_content_store2::{Config, ContentProvider, ContentWriterExt, Identifier};
use lgn_data_compiler::{
    compiler_api::{CompilationOutput, CompilerError},
    compiler_cmd::CompilerCompileCmd,
    CompiledResource,
};
use tokio::fs;

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

    Ok(deploy_remotely(
        provider,
        &source,
        PathBuf::from(resource_dir).parent().unwrap(),
        files_to_package,
    )
    .await?)
}

/// Upload the data compiler & its associated input dependencies to the CAS and create a message.
pub(crate) async fn collect_local_resources(
    executable: &Path,
    resource_dir: &Path,
    cas_local_path: &Path,
    compile_path: &ResourcePathId,
    dependencies: &[ResourcePathId],
    derived_deps: &[CompiledResource],
    build_script: &CompilerCompileCmd,
) -> Result<String, CompilerError> {
    let provider = Config::load_and_instantiate_volatile_provider()
        .await
        .map_err(|err| {
            CompilerError::RemoteExecution(format!("failed to create content provider: {}", err))
        })?;

    let files_to_package: Arc<RwLock<Vec<(String, Identifier)>>> =
        Arc::new(RwLock::new(Vec::new()));

    // Write the compiler .exe
    deploy_remotely(
        &provider,
        executable,
        executable.parent().unwrap(),
        files_to_package.clone(),
    )
    .await?;

    // Write the main resource.
    write_res(
        &provider,
        compile_path,
        resource_dir,
        files_to_package.clone(),
    )
    .await?;

    // Write the direct offline dependencies
    for dep in dependencies {
        write_res(&provider, dep, resource_dir, files_to_package.clone()).await?;
    }

    // Write the derived dependencies - not sure this is really needed
    for der_dep in derived_deps {
        let mut source = PathBuf::from(cas_local_path);
        source.push(&format!("{}", der_dep.checksum));

        deploy_remotely(
            &provider,
            &source,
            PathBuf::from(cas_local_path).parent().unwrap(),
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
    for file in files {
        let mut file_name = PathBuf::new();
        file_name.push(out_folder);
        file_name.push(&file.0);

        fs::create_dir_all(file_name.parent().unwrap()).await?;

        let mut input = provider.get_content_reader(&file.1).await?;
        let mut output = fs::File::create(&file_name).await?;

        tokio::io::copy_buf(
            &mut tokio::io::BufReader::with_capacity(10 * 1024 * 1024, &mut input),
            &mut output,
        )
        .await?;
    }
    Ok(())
}

/// Given a message, retrieve the inputs & data a compiler from the CAS, and execute the data compiler remotely.
pub(crate) async fn execute_sandbox_compiler(input_msg: &str) -> Result<String, NCError> {
    let out_folder = tempfile::tempdir()?;
    let msg: CompileMessage = serde_json::from_str(input_msg)?;

    let provider = Config::load_and_instantiate_volatile_provider()
        .await
        .map_err(|err| {
            CompilerError::RemoteExecution(format!("failed to create content provider: {}", err))
        })?;

    // Retrieve all inputs from the CAS.
    deploy_files(&provider, &msg.files_to_package, out_folder.path()).await?;

    // FIXME: Ensure there's a local CAS folder.
    let mut cas_local_path = PathBuf::from(out_folder.path());
    cas_local_path.push("temp");
    if !cas_local_path.exists() {
        fs::create_dir(cas_local_path).await?;
    }

    // Run
    let output = msg.build_script.execute_with_cwd(&out_folder)?;

    // Compress the outcome
    let output = create_resulting_archive(&output, out_folder.path()).await?;
    fs::remove_dir_all(out_folder).await?;
    Ok(output)
}

/// The incoming message from the Data-Executor worker, describing the result of a compilation.
#[derive(Serialize, Deserialize)]
pub struct CompileResultMessage {
    pub output: CompilationOutput,
    pub files_to_package: Vec<(String, Identifier)>,
}

/// Upload the results from a data compiler's output to the CAS and create a return message.
#[allow(dead_code)]
pub(crate) async fn create_resulting_archive(
    output: &CompilationOutput,
    cur_dir: &Path,
) -> Result<String, NCError> {
    let provider = Config::load_and_instantiate_volatile_provider()
        .await
        .map_err(|err| {
            CompilerError::RemoteExecution(format!("failed to create content provider: {}", err))
        })?;

    let files_to_package: Arc<RwLock<Vec<(String, Identifier)>>> =
        Arc::new(RwLock::new(Vec::new()));

    let mut cas_local_path = PathBuf::from(cur_dir);
    cas_local_path.push("temp");

    // Write the output artifacts.
    for der_dep in &output.compiled_resources {
        let mut source = cas_local_path.clone();
        source.push(&format!("{}", der_dep.checksum));

        deploy_remotely(
            &provider,
            &source,
            cas_local_path.parent().unwrap(),
            files_to_package.clone(),
        )
        .await?;
    }

    // Write the output into the message.
    let msg = CompileResultMessage {
        output: (*output).clone(),
        files_to_package: files_to_package.read().unwrap().clone(),
    };
    Ok(serde_json::to_string_pretty(&msg)?)
}
