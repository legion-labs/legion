use anyhow::{Context, Result};
use sqlx::migrate::MigrateDatabase;
use std::path::{Path, PathBuf};

pub fn get_data_directory() -> Result<PathBuf> {
    let folder =
        std::env::var("LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY").with_context(|| {
            String::from("Error reading env variable LEGION_TELEMETRY_INGESTION_SRC_DATA_DIRECTORY")
        })?;
    Ok(PathBuf::from(folder))
}

async fn create_tables(connection: &mut sqlx::AnyConnection) -> Result<()> {
    let sql = "
         CREATE TABLE processes(
                  id VARCHAR(36), 
                  exe VARCHAR(255), 
                  username VARCHAR(255), 
                  realname VARCHAR(255), 
                  computer VARCHAR(255), 
                  distro VARCHAR(255), 
                  cpu_brand VARCHAR(255), 
                  tsc_frequency BIGINT,
                  start_time VARCHAR(255));
         CREATE UNIQUE INDEX process_id on processes(id);";
    sqlx::query(sql)
        .execute(connection)
        .await
        .with_context(|| String::from("Creating table processes and its indices"))?;
    Ok(())
}

pub async fn alloc_sql_pool(data_folder: &Path) -> Result<sqlx::AnyPool> {
    let db_path = data_folder.join("telemetry.db3");
    let db_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    let new_db;
    if sqlx::Any::database_exists(&db_uri)
        .await
        .with_context(|| String::from("Searching for telemetry database"))?
    {
        new_db = false;
    } else {
        sqlx::Any::create_database(&db_uri)
            .await
            .with_context(|| String::from("Creating telemetry database"))?;
        new_db = true;
    }
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    if new_db {
        let mut connection = pool.acquire().await?;
        create_tables(&mut connection).await?;
    }
    Ok(pool)
}
