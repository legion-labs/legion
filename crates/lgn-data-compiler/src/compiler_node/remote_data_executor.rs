use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use lgn_data_compiler_remote::NCError;
use lgn_data_offline::ResourcePathId;
use zip::result::ZipResult;

use crate::{
    compiler_cmd::{CompilerCompileCmd, CompilerCompileCmdOutput},
    CompiledResource,
};

/// Create a .zip archive with the data compiler & its associated input dependencies.
pub(crate) fn collect_local_resources(
    executable: &Path,
    resource_dir: &Path,
    cas_local_path: &Path,
    compile_path: &ResourcePathId,
    dependencies: &[ResourcePathId],
    derived_deps: &[CompiledResource],
    build_script: &str,
) -> ZipResult<Vec<u8>> {
    let mut buff = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buff);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let mut add_to_zip = |full_file_path: &Path, strip_prefix: &Path| -> ZipResult<()> {
            let relative_path = full_file_path.strip_prefix(strip_prefix).unwrap();
            zip.start_file(relative_path.to_str().unwrap(), options)?;
            let buf = fs::read(&full_file_path)?;
            zip.write_all(&buf)?;
            Ok(())
        };

        // Write the compiler .exe
        add_to_zip(executable, executable.parent().unwrap())?;

        // Write the resources
        let mut write_res = |res: &ResourcePathId| -> ZipResult<()> {
            let mut source = PathBuf::from(resource_dir);
            source.push(&res.source_resource().id.resource_path());

            add_to_zip(&source, PathBuf::from(resource_dir).parent().unwrap())
        };

        write_res(compile_path)?;

        // Write the direct offline dependencies
        for dep in dependencies {
            write_res(dep)?;
        }

        // Write the derived dependencies - not sure this is really needed
        for der_dep in derived_deps {
            let mut source = PathBuf::from(cas_local_path);
            source.push(&format!("{}", der_dep.checksum));

            add_to_zip(&source, PathBuf::from(cas_local_path).parent().unwrap())?;
        }

        // Write the build script
        zip.start_file("build.json", options)?;
        zip.write_all(build_script.as_bytes())?;

        zip.finish()?;
    }
    Ok(buff.into_inner())
}

/// Create a .zip archive from the data compiler's output.
pub(crate) fn create_resulting_archive(
    stdout: &CompilerCompileCmdOutput,
    cur_dir: &Path,
) -> Result<Vec<u8>, NCError> {
    let mut buff = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buff);
        let options =
            zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        let mut add_to_zip = |full_file_path: &Path, strip_prefix: &Path| -> ZipResult<()> {
            let relative_path = full_file_path.strip_prefix(strip_prefix).unwrap();
            zip.start_file(relative_path.to_str().unwrap(), options)?;
            let buf = fs::read(&full_file_path)?;
            zip.write_all(&buf)?;
            Ok(())
        };

        let mut cas_local_path = PathBuf::from(cur_dir);
        cas_local_path.push("temp");

        // Write the output artifacts.
        for der_dep in &stdout.compiled_resources {
            let mut source = cas_local_path.clone();
            source.push(&format!("{}", der_dep.checksum));

            add_to_zip(&source, cas_local_path.parent().unwrap())?;
        }

        // Write the output.
        zip.start_file("output.json", options)?;
        zip.write_all(stdout.to_string().as_bytes())?;

        zip.finish()?;
    }
    Ok(buff.into_inner())
}

/// Uncompress and execute a data compiler remotely.
pub(crate) fn execute_sandbox_compiler(input_archive: &[u8]) -> Result<Vec<u8>, NCError> {
    let out_folder = tempfile::tempdir()?;

    // Write the archive to a file for debugging.
    /*let local_path = out_folder.path();
    let mut file_archive = PathBuf::from(local_path);
    file_archive.push(format!("{:#x}.zip", rand::random::<u64>()));
    fs::write(file_archive, &input_archive)?;*/

    // Uncompress
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&input_archive))?;
    archive.extract(&out_folder)?;

    // Ensure there's a CAS folder.
    let mut cas_local_path = PathBuf::from(out_folder.path());
    cas_local_path.push("temp");
    if !cas_local_path.exists() {
        fs::create_dir(cas_local_path)?;
    }

    // Build file
    let mut build_file_name = PathBuf::from(out_folder.path());
    build_file_name.push("build.json");

    let build_file = CompilerCompileCmd::from_slice(&fs::read_to_string(build_file_name)?);

    // Run
    let output = build_file.execute_with_cwd(&out_folder)?;

    // Compress the outcome
    let output = create_resulting_archive(&output, out_folder.path())?;
    fs::remove_dir_all(out_folder)?;
    Ok(output)
}
