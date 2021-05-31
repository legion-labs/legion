use crate::*;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::io::Write;
use std::path::Path;

fn write_blob(file_path: &Path, contents: &[u8]) -> Result<(), String> {
    if fs::metadata(file_path).is_ok() {
        //blob already exists
        return Ok(());
    }

    match std::fs::File::create(file_path) {
        Err(e) => {
            return Err(format!("Error creating file {:?}: {}", file_path, e));
        }
        Ok(output_file) => match lz4::EncoderBuilder::new().level(10).build(output_file) {
            Err(e) => return Err(format!("Error building lz4 encoder: {}", e)),
            Ok(mut encoder) => {
                if let Err(e) = encoder.write(contents) {
                    return Err(format!("Error writing to lz4 encoder: {}", e));
                }
                if let (_w, Err(e)) = encoder.finish() {
                    return Err(format!("Error closing lz4 encoder: {}", e));
                }
                Ok(())
            }
        },
    }
}

fn upload_localy_edited_blobs(
    workspace_root: &Path,
    workspace_spec: &Workspace,
) -> Result<(), String> {
    let blob_dir = Path::new(&workspace_spec.repository).join("blobs");
    for local_change in find_local_changes(workspace_root)? {
        let workspace_path = workspace_root.join(local_change.relative_path);
        //todo: support binary files
        let local_file_contents = read_file(&workspace_path)?;
        let hash = format!("{:X}", Sha256::digest(local_file_contents.as_bytes()));
        write_blob(&blob_dir.join(hash), local_file_contents.as_bytes())?;
    }
    Ok(())
}

pub fn commit(_message: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    upload_localy_edited_blobs(workspace_root, &workspace_spec)?;

    //todo: build trees
    //todo: save commit
    //todo: make local files read only
    //todo: clear local changes
    //todo: update branch

    Ok(())
}
