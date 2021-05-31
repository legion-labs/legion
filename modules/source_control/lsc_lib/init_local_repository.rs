use crate::*;
use std::fs;
use std::path::Path;

pub fn init_local_repository(directory: &Path) -> Result<(), String> {
    if fs::metadata(directory).is_ok() {
        return Err(format!("{} already exists", directory.display()));
    }
    if let Err(e) = fs::create_dir_all(directory.join("trees")) {
        return Err(format!("Error creating trees directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(directory.join("commits")) {
        return Err(format!("Error creating commits directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(directory.join("blobs")) {
        return Err(format!("Error creating blobs directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(directory.join("branches")) {
        return Err(format!("Error creating branches directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(directory.join("workspaces")) {
        return Err(format!("Error creating workspaces directory: {}", e));
    }
    if let Err(e) = fs::create_dir_all(directory.join("locks")) {
        return Err(format!("Error creating locks directory: {}", e));
    }

    let root_tree = Tree::empty();
    let root_hash = root_tree.hash();
    save_tree(directory, &root_tree, &root_hash)?;

    let initial_commit = Commit::new(
        whoami::username(),
        String::from("initial commit"),
        Vec::new(),
        root_hash,
        Vec::new(),
    );
    save_commit(directory, &initial_commit)?;

    let main_branch = Branch::new(String::from("main"), initial_commit.id);
    save_branch_to_repo(directory, &main_branch)?;

    Ok(())
}
