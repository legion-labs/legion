use thiserror::Error;

use crate::FourCC;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    InvalidData(&'static str),
    #[error("{0} not found")]
    BoxNotFound(FourCC),
    #[error("{0} and {1} not found")]
    Box2NotFound(FourCC, FourCC),
    #[error("trak[{0}] not found")]
    TrakNotFound(u32),
    #[error("trak[{0}].{1} not found")]
    BoxInTrakNotFound(u32, FourCC),
    #[error("traf[{0}].{1} not found")]
    BoxInTrafNotFound(u32, FourCC),
    #[error("trak[{0}].stbl.{1} not found")]
    BoxInStblNotFound(u32, FourCC),
    #[error("trak[{0}].stbl.{1}.entry[{2}] not found")]
    EntryInStblNotFound(u32, FourCC, u32),
    #[error("traf[{0}].trun.{1}.entry[{2}] not found")]
    EntryInTrunNotFound(u32, FourCC, u32),
}

pub type Result<T> = anyhow::Result<T, Error>;
