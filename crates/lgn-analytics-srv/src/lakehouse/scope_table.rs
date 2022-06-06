use std::path::Path;

use super::{
    column::{Column, TableColumn},
    parquet_buffer::ParquetBufferWriter,
};
use crate::scope::ScopeHashMap;
use anyhow::{Context, Result};
use lgn_telemetry_proto::analytics::ScopeDesc;
use parquet::data_type::ByteArray;
use std::io::Write;

#[derive(Debug)]
pub struct ScopeRowGroup {
    pub hashes: Column<i32>,
    pub names: Column<ByteArray>,
    pub filenames: Column<ByteArray>,
    pub lines: Column<i32>,
}

impl ScopeRowGroup {
    pub fn new() -> Self {
        Self {
            hashes: Column::new(),
            names: Column::new(),
            filenames: Column::new(),
            lines: Column::new(),
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn append(&mut self, row: &ScopeDesc) {
        self.hashes.append(row.hash as i32);
        self.names.append(ByteArray::from(row.name.as_str()));
        self.filenames.append(ByteArray::from(row.filename.as_str()));
        self.lines.append(row.line as i32);
    }

    pub fn get_columns(&self) -> Vec<&dyn TableColumn> {
        vec![&self.hashes, &self.names, &self.filenames, &self.lines]
    }
}

pub fn write_scopes_parquet(scopes: &ScopeHashMap, parquet_path: &Path) -> Result<()> {
    let mut rows = ScopeRowGroup::new();
    for (_k, v) in scopes.iter() {
        rows.append(v);
    }
    let schema = "message schema {
    REQUIRED INT32 hash;
    REQUIRED BYTE_ARRAY name;
    REQUIRED BYTE_ARRAY filename;
    REQUIRED INT32 line;
  }
";
    let mut writer = ParquetBufferWriter::create(schema)?;
    writer.write_row_group(&rows.get_columns())?;
    let buffer = writer.close()?;
    //todo: extraire
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(parquet_path)
        .with_context(|| format!("creating file {}", parquet_path.display()))?;
    file.write_all(buffer.as_ref().get_ref())?;
    Ok(())
}
