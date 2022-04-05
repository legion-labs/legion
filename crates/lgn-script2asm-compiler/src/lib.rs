use async_trait::async_trait;
use curl::easy::Easy;
use std::{
    env::temp_dir,
    ffi::OsString,
    fs::{self, File},
    io::Write,
    path::Path,
};

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::Transform;
use lgn_data_runtime::{AssetRegistryOptions, Resource, ResourceTypeAndId};
use lgn_scripting::offline as offline_data;
use lgn_scripting::runtime as runtime_data;
use lgn_scripting::ScriptType;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(offline_data::Script::TYPE, runtime_data::Script::TYPE),
    compiler_creator: || Box::new(Script2AsmCompiler {}),
};

struct Script2AsmCompiler();

impl Script2AsmCompiler {
    #[allow(unused_variables, clippy::unnecessary_wraps)]
    fn get_compiled_script(
        resource_id: &ResourceTypeAndId,
        resource: &offline_data::Script,
    ) -> Result<runtime_data::Script, CompilerError> {
        #[cfg(target_os = "windows")]
        {
            // Avoid packaging mun.exe in the repo
            let mun_exe = Self::make_mun_available()?;
            let temp_crate = {
                let mut temp_crate = temp_dir();
                temp_crate.push(resource_id.id.to_string());
                temp_crate
            };
            if !temp_crate.is_dir() {
                std::process::Command::new(mun_exe.as_os_str())
                    .arg("new")
                    .arg(temp_crate.to_str().unwrap())
                    .spawn()
                    .map_err(|err| {
                        CompilerError::CompilationError(format!("Cannot start 'mun new': {}", err))
                    })?
                    .wait()
                    .map_err(|err| {
                        CompilerError::CompilationError(format!("Cannot start 'mun new': {}", err))
                    })?;
            }
            {
                let mut src_path = std::path::PathBuf::from(&temp_crate);
                src_path.push("src");
                src_path.push("mod.mun");
                fs::write(src_path.clone(), resource.script.as_bytes()).map_err(|err| {
                    CompilerError::CompilationError(format!(
                        "Failed save script {}: {}",
                        src_path.as_path().display(),
                        err
                    ))
                })?;

                let mut toml_path = temp_crate.clone();
                toml_path.push("mun.toml");
                std::process::Command::new(mun_exe.as_os_str())
                    .arg("build")
                    .arg("--manifest-path")
                    .arg(toml_path.to_str().unwrap())
                    .spawn()
                    .map_err(|err| {
                        CompilerError::CompilationError(format!(
                            "Cannot start 'mun build': {}",
                            err
                        ))
                    })?
                    .wait()
                    .map_err(|err| {
                        CompilerError::CompilationError(format!(
                            "Cannot build mum project: {}",
                            err
                        ))
                    })?;
            }
            Ok(runtime_data::Script {
                script_type: ScriptType::Mun,
                compiled_script: {
                    let mut src_path = std::path::PathBuf::from(&temp_crate);
                    src_path.push("target");
                    src_path.push("mod.munlib");
                    fs::read(src_path.clone()).map_err(|err| {
                        CompilerError::CompilationError(format!(
                            "Failed load script {}: {}",
                            src_path.as_path().display(),
                            err
                        ))
                    })?
                },
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(runtime_data::Script {
                script_type: ScriptType::Mun,
                compiled_script: Vec::new(),
            })
        }
    }

    // mun.exe is big, download it locally to avoid packaging it in the repo.
    // This is a temporary situation until we decide what to do with the scripting language.
    #[allow(dead_code)]
    fn make_mun_available() -> Result<OsString, CompilerError> {
        let mut mun_local_exe = temp_dir();
        mun_local_exe.push("mun.exe");
        if !mun_local_exe.is_file() {
            fn download(url: &str, target_path: &Path) -> Result<(), CompilerError> {
                let mut handle = Easy::new();
                let mut file = File::create(target_path).map_err(|err| {
                    CompilerError::CompilationError(format!("Failed to retrieve mun: {}", err))
                })?;

                handle.url(url).unwrap();
                handle
                    .follow_location(true)
                    .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

                let mut transfer = handle.transfer();
                {
                    transfer
                        .write_function(|new_data| {
                            file.write_all(new_data).unwrap();
                            Ok(new_data.len())
                        })
                        .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

                    transfer
                        .perform()
                        .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

                    drop(transfer);
                }
                Ok(())
            }
            let mut mun_zip = temp_dir();
            mun_zip.push("mun.zip");
            download(
                "https://github.com/mun-lang/mun/releases/download/v0.3.0/mun-win64-v0.3.0.zip",
                &mun_zip,
            )?;

            // uncompress
            let zip_file = fs::File::open(&mun_zip)
                .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

            let mut archive = zip::ZipArchive::new(zip_file)
                .map_err(|err| CompilerError::CompilationError(err.to_string()))?;
            archive
                .extract(temp_dir())
                .map_err(|err| CompilerError::CompilationError(err.to_string()))?;
        }
        Ok(mun_local_exe.into_os_string())
    }
}

#[async_trait]
impl Compiler for Script2AsmCompiler {
    async fn init(&self, options: AssetRegistryOptions) -> AssetRegistryOptions {
        options.add_loader::<offline_data::Script>()
    }

    async fn hash(
        &self,
        code: &'static str,
        data: &'static str,
        env: &CompilationEnv,
    ) -> CompilerHash {
        hash_code_and_data(code, data, env)
    }

    #[lgn_tracing::span_fn]
    async fn compile(
        &self,
        mut context: CompilerContext<'_>,
    ) -> Result<CompilationOutput, CompilerError> {
        let resources = context.registry();

        let result_buffer = {
            let resource = resources
                .load_async::<offline_data::Script>(context.source.resource_id())
                .await;

            let resource = resource.get(&resources).ok_or_else(|| {
                CompilerError::CompilationError(format!(
                    "Failed to retrieve resource {}",
                    context.source.resource_id()
                ))
            })?;

            let runtime_script = match resource.script_type {
                ScriptType::Mun => {
                    Self::get_compiled_script(&context.source.resource_id(), &resource)?
                }
                _ => runtime_data::Script {
                    script_type: resource.script_type,
                    compiled_script: resource.script.as_bytes().to_vec(),
                },
            };
            bincode::serialize(&runtime_script).map_err(|err| {
                CompilerError::CompilationError(format!("Failed to bincode script: {}", err))
            })?
        };

        let asset = context
            .store(&result_buffer, context.target_unnamed.clone())
            .await?;

        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
