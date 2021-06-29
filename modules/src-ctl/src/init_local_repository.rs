use crate::*;
use std::fs;
use std::path::Path;

pub fn init_local_repository(directory: &Path) -> Result<(), String> {
    if fs::metadata(directory).is_ok() {
        return Err(format!("{} already exists", directory.display()));
    }

    if let Err(e) = fs::create_dir_all(&directory) {
        return Err(format!("Error creating repository directory: {}", e));
    }

    let repo_connection = Connection::new(directory)?;
    init_forest_database(&repo_connection)?;

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

    let lock_domain_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = fs::create_dir_all(directory.join(format!("lock_domains/{}", lock_domain_id))) {
        return Err(format!("Error creating locks directory: {}", e));
    }

    let root_tree = Tree::empty();
    let root_hash = root_tree.hash();
    save_tree(&repo_connection, &root_tree, &root_hash)?;

    let id = uuid::Uuid::new_v4().to_string();
    let initial_commit = Commit::new(
        id,
        whoami::username(),
        String::from("initial commit"),
        Vec::new(),
        root_hash,
        Vec::new(),
    );
    save_commit(directory, &initial_commit)?;

    let main_branch = Branch::new(
        String::from("main"),
        initial_commit.id,
        String::new(),
        lock_domain_id,
    );
    save_branch_to_repo(directory, &main_branch)?;

    Ok(())
}
