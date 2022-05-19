use anyhow::{Context, Result};
use parquet::{column::writer::ColumnWriter, file::writer::RowGroupWriter};

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

impl Column<i32> {
    pub fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
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

impl Column<i64> {
    pub fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
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

impl Column<f64> {
    pub fn write_batch(&self, row_group_writer: &mut dyn RowGroupWriter) -> Result<()> {
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
