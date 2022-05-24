use std::{path::PathBuf, sync::Arc};

use crate::jit_lakehouse::JitLakehouse;
use crate::span_table::{make_span_table, write_parquet};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lgn_analytics::{prelude::*, time::ConvertTicks};
use lgn_blob_storage::BlobStorage;
use lgn_tracing::prelude::*;
use tokio::fs;

pub struct LocalJitLakehouse {
    pool: sqlx::any::AnyPool,
    blob_storage: Arc<dyn BlobStorage>,
    tables_path: PathBuf,
}

impl LocalJitLakehouse {
    pub fn new(
        pool: sqlx::any::AnyPool,
        blob_storage: Arc<dyn BlobStorage>,
        tables_path: PathBuf,
    ) -> Self {
        Self {
            pool,
            blob_storage,
            tables_path,
        }
    }
}

#[async_trait]
impl JitLakehouse for LocalJitLakehouse {
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
        async_span_scope!("build_timeline_tables");
        let mut connection = self.pool.acquire().await?;
        let process = find_process(&mut connection, process_id).await?;
        let convert_ticks = ConvertTicks::new(&process);
        let table = make_span_table(
            &mut connection,
            self.blob_storage.clone(),
            process_id,
            &convert_ticks,
        )
        .await?;
        let spans_table_path = self.tables_path.join(process_id).join("spans");
        fs::create_dir_all(&spans_table_path)
            .await
            .with_context(|| format!("creating folder {}", spans_table_path.display()))?;
        //todo: create file at the beginning and write to it as the blocks arrive
        write_parquet(&spans_table_path.join("spans.parquet"), &table)?;
        Ok(())
    }
}
