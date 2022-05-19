use std::path::Path;
use std::{path::PathBuf, sync::Arc};

use crate::span_table::make_span_table;
use crate::{
    jit_lakehouse::JitLakehouse,
    span_table::SpanTable
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use lgn_analytics::{prelude::*, time::ConvertTicks};
use lgn_blob_storage::BlobStorage;
use lgn_tracing::prelude::*;
use parquet::column::writer::ColumnWriter;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::FileWriter;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;
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


#[span_fn]
fn write_parquet(file_path: &Path, spans: &SpanTable) -> Result<()> {
    let message_type = "
  message schema {
    REQUIRED INT32 hash;
    REQUIRED INT32 depth;
    REQUIRED DOUBLE begin_ms;
    REQUIRED DOUBLE end_ms;
  }
";
    let schema =
        Arc::new(parse_message_type(message_type).with_context(|| "parsing spans schema")?);
    let props = Arc::new(WriterProperties::builder().build());
    let file = std::fs::File::create(file_path)
        .with_context(|| format!("creating file {}", file_path.display()))?;
    let mut writer = SerializedFileWriter::new(file, schema, props)
        .with_context(|| "creating parquet writer")?;
    let mut row_group_writer = writer
        .next_row_group()
        .with_context(|| "creating row group writer")?;
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::Int32ColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.hashes.values, None, None)
                .with_context(|| "writing hash batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::Int32ColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.depths.values, None, None)
                .with_context(|| "writing depth batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::DoubleColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.begins.values, None, None)
                .with_context(|| "writing begins batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    if let Some(mut col_writer) = row_group_writer
        .next_column()
        .with_context(|| "creating column writer")?
    {
        if let ColumnWriter::DoubleColumnWriter(writer_impl) = &mut col_writer {
            writer_impl
                .write_batch(&spans.ends.values, None, None)
                .with_context(|| "writing ends batch")?;
        }
        row_group_writer
            .close_column(col_writer)
            .with_context(|| "closing column")?;
    }
    writer
        .close_row_group(row_group_writer)
        .with_context(|| "closing row group")?;
    writer.close().with_context(|| "closing parquet writer")?;
    Ok(())
}

#[async_trait]
impl JitLakehouse for LocalJitLakehouse {
    #[span_fn]
    async fn build_timeline_tables(&self, process_id: &str) -> Result<()> {
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
        write_parquet(&spans_table_path.join("spans.parquet"), &table)?;
        Ok(())
    }
}
