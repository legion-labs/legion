use crate::*;
use std::fs;
use std::path::Path;

pub fn init_repo_database(repo_connection: &mut RepositoryConnection) -> Result<(), String> {
    init_commit_database(repo_connection)?;
    init_forest_database(repo_connection)?;
    init_branch_database(repo_connection)?;
    init_workspace_database(repo_connection)?;
    init_lock_database(repo_connection)?;
    Ok(())
}

pub fn push_init_repo_data(repo_connection: &mut RepositoryConnection) -> Result<(), String> {
    let lock_domain_id = uuid::Uuid::new_v4().to_string();
    let root_tree = Tree::empty();
    let root_hash = root_tree.hash();
    save_tree(repo_connection, &root_tree, &root_hash)?;

    let id = uuid::Uuid::new_v4().to_string();
    let initial_commit = Commit::new(
        id,
        whoami::username(),
        String::from("initial commit"),
        Vec::new(),
        root_hash,
        Vec::new(),
    );
    save_commit(repo_connection, &initial_commit)?;

    let main_branch = Branch::new(
        String::from("main"),
        initial_commit.id,
        String::new(),
        lock_domain_id,
    );
    save_new_branch_to_repo(repo_connection, &main_branch)?;
    Ok(())
}

pub fn init_local_repository(directory: &Path) -> Result<RepositoryAddr, String> {
    if fs::metadata(directory).is_ok() {
        return Err(format!("{} already exists", directory.display()));
    }

    if let Err(e) = fs::create_dir_all(&directory) {
        return Err(format!("Error creating repository directory: {}", e));
    }

    let db_path = make_path_absolute(&directory.join("repo.db3"));
    let repo_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    create_database(&repo_uri)?;

    let blob_dir = make_path_absolute(&directory.join("blobs"));
    if let Err(e) = fs::create_dir_all(&blob_dir) {
        return Err(format!("Error creating blobs directory: {}", e));
    }

    let addr = RepositoryAddr {
        repo_uri,
        blob_store: BlobStorageSpec::LocalDirectory(blob_dir),
    };
    let mut repo_connection = RepositoryConnection::new(&addr)?;

    init_repo_database(&mut repo_connection)?;
    push_init_repo_data(&mut repo_connection)?;

    Ok(addr)
}
