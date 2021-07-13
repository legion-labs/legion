use crate::*;
use std::fs;
use std::path::Path;

pub fn init_local_repository(directory: &Path) -> Result<RepositoryAddr, String> {
    if fs::metadata(directory).is_ok() {
        return Err(format!("{} already exists", directory.display()));
    }

    if let Err(e) = fs::create_dir_all(&directory) {
        return Err(format!("Error creating repository directory: {}", e));
    }

    let db_path = make_path_absolute(&directory.join("repo.db3"));
    let repo_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    create_sqlite_database(&repo_uri)?;

    let blob_dir = make_path_absolute(&directory.join("blobs"));
    if let Err(e) = fs::create_dir_all(&blob_dir) {
        return Err(format!("Error creating blobs directory: {}", e));
    }
    let blob_uri = format!("file://{}", blob_dir.to_str().unwrap().replace("\\", "/"));

    let addr = RepositoryAddr{ repo_uri: repo_uri.clone(), blob_uri };
    let mut repo_connection = RepositoryConnection::new(&addr)?;
    init_commit_database(&mut repo_connection)?;
    init_forest_database(&mut repo_connection)?;
    init_branch_database(&mut repo_connection)?;
    init_workspace_database(&mut repo_connection)?;
    init_lock_database(&mut repo_connection)?;


    let lock_domain_id = uuid::Uuid::new_v4().to_string();
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
    Ok(addr)
}
