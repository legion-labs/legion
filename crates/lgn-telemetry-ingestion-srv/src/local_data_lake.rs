use anyhow::{Context, Result};
use lgn_blob_storage::LocalBlobStorage;
use lgn_tracing::info;
use sqlx::migrate::MigrateDatabase;
use std::path::PathBuf;

use crate::{
    ingestion_service::IngestionService,
    local_telemetry_db::{create_tables, read_schema_version},
};

async fn migrate_db(connection: &mut sqlx::AnyConnection) -> Result<()> {
    let mut current_version = read_schema_version(connection).await;
    if 0 == current_version {
        info!("creating v1 schema");
        create_tables(connection).await?;
        current_version = read_schema_version(connection).await;
    }
    assert_eq!(current_version, 1);
    Ok(())
}

pub async fn connect_to_local_data_lake(path: PathBuf) -> Result<IngestionService> {
    let blocks_folder = path.join("blobs");
    let blob_storage = LocalBlobStorage::new(blocks_folder).await?;
    let db_path = path.join("telemetry.db3");
    let db_uri = format!("sqlite://{}", db_path.to_str().unwrap().replace("\\", "/"));
    if !sqlx::Any::database_exists(&db_uri)
        .await
        .with_context(|| String::from("Searching for telemetry database"))?
    {
        sqlx::Any::create_database(&db_uri)
            .await
            .with_context(|| String::from("Creating telemetry database"))?;
    }
    let pool = sqlx::any::AnyPoolOptions::new()
        .connect(&db_uri)
        .await
        .with_context(|| String::from("Connecting to telemetry database"))?;
    let mut connection = pool.acquire().await?;
    migrate_db(&mut connection).await?;
    Ok(IngestionService::new(pool, Box::new(blob_storage)))
}
