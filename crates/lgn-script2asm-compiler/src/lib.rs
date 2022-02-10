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
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::Transform;
use lgn_data_runtime::{AssetRegistryOptions, Resource, ResourceTypeAndId};
use lgn_scripting::offline as offline_data;
use lgn_scripting::runtime as runtime_data;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(offline_data::Script::TYPE, runtime_data::Script::TYPE),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(options: AssetRegistryOptions) -> AssetRegistryOptions {
    options.add_loader::<offline_data::Script>()
}

#[allow(unused_variables)]
fn get_compiled_script(
    resource_id: &ResourceTypeAndId,
    resource: &offline_data::Script,
) -> runtime_data::Script {
    #[cfg(target_os = "windows")]
    {
        // Avoid packaging mun.exe in the repo
        let mun_exe = make_mun_available();
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
                .expect("Cannot start 'mun new'")
                .wait()
                .expect("Cannot create mun project");
        }
        {
            let mut src_path = std::path::PathBuf::from(&temp_crate);
            src_path.push("src");
            src_path.push("mod.mun");
            //println!("{:?}", &src_path);
            fs::write(src_path, resource.script.as_bytes()).unwrap();

            let mut toml_path = temp_crate.clone();
            toml_path.push("mun.toml");
            std::process::Command::new(mun_exe.as_os_str())
                .arg("build")
                .arg("--manifest-path")
                .arg(toml_path.to_str().unwrap())
                .spawn()
                .expect("Cannot start 'mun build'")
                .wait()
                .expect("Cannot build mun project");
        }
        runtime_data::Script {
            script_type: 1, // Mun
            compiled_script: {
                let mut src_path = std::path::PathBuf::from(&temp_crate);
                src_path.push("target");
                src_path.push("mod.munlib");
                //println!("{:?}", &src_path);
                fs::read(src_path).unwrap()
            },
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        runtime_data::Script {
            script_type: 1, // Mun
            compiled_script: Vec::new(),
        }
    }
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource = resources.load_sync::<offline_data::Script>(context.source.resource_id());
    let resource = resource.get(&resources).unwrap();

    let runtime_script = match resource.script_type {
        1 => get_compiled_script(&context.source.resource_id(), &resource),
        _ => runtime_data::Script {
            script_type: resource.script_type,
            compiled_script: resource.script.as_bytes().to_vec(),
        },
    };
    let result_buffer = bincode::serialize(&runtime_script).unwrap();

    let asset = context.store(&result_buffer, context.target_unnamed.clone())?;

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}

// mun.exe is big, download it locally to avoid packaging it in the repo.
// This is a temporary situation until we decide what to do with the scripting language.
#[allow(dead_code)]
fn make_mun_available() -> OsString {
    let mut mun_local_exe = temp_dir();
    mun_local_exe.push("mun.exe");
    if !mun_local_exe.is_file() {
        fn download(url: &str, target_path: &Path) {
            let mut handle = Easy::new();
            let mut file = File::create(target_path).unwrap();

            handle.url(url).unwrap();
            handle.follow_location(true).unwrap();

            let mut transfer = handle.transfer();
            {
                transfer
                    .write_function(|new_data| {
                        file.write_all(new_data).unwrap();
                        Ok(new_data.len())
                    })
                    .unwrap();
                transfer.perform().unwrap();
                drop(transfer);
            }
        }
        let mut mun_zip = temp_dir();
        mun_zip.push("mun.zip");
        download(
            "https://github.com/mun-lang/mun/releases/download/v0.3.0/mun-win64-v0.3.0.zip",
            &mun_zip,
        );

        // uncompress
        let zip_file = fs::File::open(&mun_zip).unwrap();
        let mut archive = zip::ZipArchive::new(zip_file).unwrap();
        archive.extract(temp_dir()).unwrap();
    }
    mun_local_exe.into_os_string()
}
