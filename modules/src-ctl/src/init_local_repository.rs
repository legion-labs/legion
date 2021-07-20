use crate::*;
use std::fs;
use std::path::Path;

pub fn init_repo_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    init_config_database(sql_connection)?;
    init_commit_database(sql_connection)?;
    init_forest_database(sql_connection)?;
    init_branch_database(sql_connection)?;
    init_workspace_database(sql_connection)?;
    init_lock_database(sql_connection)?;
    Ok(())
}

pub fn push_init_repo_data(
    sql_connection: &mut sqlx::AnyConnection,
    self_uri: &str,
    blob_storage: &BlobStorageSpec,
) -> Result<(), String> {
    insert_config(sql_connection, self_uri, blob_storage)?;

    let bogus_blob_cache = std::path::PathBuf::new();
    let mut repo_connection = RepositoryConnection::new(self_uri, bogus_blob_cache)?;
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
    Ok(())
}

pub fn init_local_repository(directory: &Path) -> Result<String, String> {
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

    let mut sql_connection = connect(&repo_uri)?;
    init_repo_database(&mut sql_connection)?;
    push_init_repo_data(
        &mut sql_connection,
        &repo_uri,
        &BlobStorageSpec::LocalDirectory(blob_dir),
    )?;
    Ok(repo_uri)
}
