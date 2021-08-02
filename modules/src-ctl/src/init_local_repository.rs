use crate::sql_repository_query::SqlRepositoryQuery;
use crate::{sql::*, *};
use futures::executor::block_on;
use std::fs;
use std::path::Path;

pub fn init_repo_database(
    pool: &sqlx::AnyPool,
    self_uri: &str,
    blob_storage: &BlobStorageSpec,
) -> Result<(), String> {
    let mut sql_connection = block_on(pool.acquire()).unwrap(); //todo, fix
    init_config_database(&mut sql_connection)?;
    init_commit_database(&mut sql_connection)?;
    init_forest_database(&mut sql_connection)?;
    init_branch_database(&mut sql_connection)?;
    init_workspace_database(&mut sql_connection)?;
    init_lock_database(&mut sql_connection)?;

    insert_config(&mut sql_connection, self_uri, blob_storage)?;
    Ok(())
}

pub async fn push_init_repo_data(self_uri: &str) -> Result<(), String> {
    let query = SqlRepositoryQuery::new(self_uri)?;
    let lock_domain_id = uuid::Uuid::new_v4().to_string();
    let root_tree = Tree::empty();
    let root_hash = root_tree.hash();
    query.save_tree(&root_tree, &root_hash).await?;

    let id = uuid::Uuid::new_v4().to_string();
    let initial_commit = Commit::new(
        id,
        whoami::username(),
        String::from("initial commit"),
        Vec::new(),
        root_hash,
        Vec::new(),
    );
    query.insert_commit(&initial_commit).await?;

    let main_branch = Branch::new(
        String::from("main"),
        initial_commit.id,
        String::new(),
        lock_domain_id,
    );
    query.insert_branch(&main_branch).await?;
    Ok(())
}

pub async fn init_local_repository(directory: &Path) -> Result<String, String> {
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

    let pool = alloc_sql_pool(&repo_uri)?;
    init_repo_database(&pool, &repo_uri, &BlobStorageSpec::LocalDirectory(blob_dir))?;
    push_init_repo_data(&repo_uri).await?;
    Ok(repo_uri)
}
