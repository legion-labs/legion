use anyhow::{Context, Result};
use lgn_tracing::prelude::*;
use parquet::{column::writer::ColumnWriter, file::writer::RowGroupWriter};
use parquet::data_type::ByteArray;

pub trait TableColumn {
    fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()>;
}

#[derive(Debug)]
pub struct Column<T> {
    pub values: Vec<T>,
}

impl<T> Column<T> {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn append(&mut self, v: T) {
        self.values.push(v);
    }
}

impl TableColumn for Column<i32> {
    #[span_fn]
    fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
        if let Some(mut col_writer) = row_group_writer
            .next_column()
            .with_context(|| "creating column writer")?
        {
            if let ColumnWriter::Int32ColumnWriter(writer_impl) = &mut col_writer {
                writer_impl
                    .write_batch(&self.values, None, None)
                    .with_context(|| "writing i32 batch")?;
            }
            row_group_writer
                .close_column(col_writer)
                .with_context(|| "closing column")?;
        }
        Ok(())
    }
}

impl TableColumn for Column<i64> {
    #[span_fn]
    fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
        if let Some(mut col_writer) = row_group_writer
            .next_column()
            .with_context(|| "creating column writer")?
        {
            if let ColumnWriter::Int64ColumnWriter(writer_impl) = &mut col_writer {
                writer_impl
                    .write_batch(&self.values, None, None)
                    .with_context(|| "writing i64 batch")?;
            }
            row_group_writer
                .close_column(col_writer)
                .with_context(|| "closing column")?;
        }
        Ok(())
    }
}

impl TableColumn for Column<f64> {
    #[span_fn]
    fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
        if let Some(mut col_writer) = row_group_writer
            .next_column()
            .with_context(|| "creating column writer")?
        {
            if let ColumnWriter::DoubleColumnWriter(writer_impl) = &mut col_writer {
                writer_impl
                    .write_batch(&self.values, None, None)
                    .with_context(|| "writing f64 batch")?;
            }
            row_group_writer
                .close_column(col_writer)
                .with_context(|| "closing column")?;
        }
        Ok(())
    }
}

impl TableColumn for Column<ByteArray> {
    #[span_fn]
    fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
        if let Some(mut col_writer) = row_group_writer
            .next_column()
            .with_context(|| "creating column writer")?
        {
            if let ColumnWriter::ByteArrayColumnWriter(writer_impl) = &mut col_writer {
                writer_impl
                    .write_batch(&self.values, None, None)
                    .with_context(|| "writing string batch")?;
            }
            row_group_writer
                .close_column(col_writer)
                .with_context(|| "closing column")?;
        }
        Ok(())
    }
}
