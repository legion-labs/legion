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

    create_sqlite_repo_database(directory)?;

    let mut repo_connection = RepositoryConnection::new(directory.to_str().unwrap())?;
    init_commit_database(&mut repo_connection)?;
    init_forest_database(&mut repo_connection)?;
    init_branch_database(&mut repo_connection)?;
    init_workspace_database(&mut repo_connection)?;

    if let Err(e) = fs::create_dir_all(directory.join("blobs")) {
        return Err(format!("Error creating blobs directory: {}", e));
    }

    let lock_domain_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = fs::create_dir_all(directory.join(format!("lock_domains/{}", lock_domain_id))) {
        return Err(format!("Error creating locks directory: {}", e));
    }

    let root_tree = Tree::empty();
    let root_hash = root_tree.hash();
    save_tree(&mut repo_connection, &root_tree, &root_hash)?;

    let id = uuid::Uuid::new_v4().to_string();
    let initial_commit = Commit::new(
        id,
        whoami::username(),
        String::from("initial commit"),
        Vec::new(),
        root_hash,
        Vec::new(),
    );
    save_commit(&mut repo_connection, &initial_commit)?;

    let main_branch = Branch::new(
        String::from("main"),
        initial_commit.id,
        String::new(),
        lock_domain_id,
    );
    save_new_branch_to_repo(&mut repo_connection, &main_branch)?;

    Ok(())
}
