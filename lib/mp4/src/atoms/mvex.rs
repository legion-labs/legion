use std::io::{Read, Seek, SeekFrom, Write};

use serde::Serialize;

use super::mehd::MehdAtom;
use super::trex::TrexAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

/// Movie Extends Header Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MvexAtom {
    pub mehd: Option<MehdAtom>,
    pub trex: TrexAtom,
}

impl Atom for MvexAtom {
    const FOUR_CC: FourCC = FourCC::new(b"mvex");

    fn size(&self) -> u64 {
        HEADER_SIZE + self.mehd.as_ref().map_or(0, MehdAtom::size) + self.trex.size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MvexAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mehd = None;
        let mut trex = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                MehdAtom::FOUR_CC => {
                    mehd = Some(MehdAtom::read_atom(reader, s)?);
                }
                TrexAtom::FOUR_CC => {
                    trex = Some(TrexAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if trex.is_none() {
            return Err(Error::BoxNotFound(TrexAtom::FOUR_CC));
        }
        let trex = trex.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { mehd, trex })
    }
}

impl<W: Write> WriteAtom<&mut W> for MvexAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        if let Some(mehd) = &self.mehd {
            mehd.write_atom(writer)?;
        }
        self.trex.write_atom(writer)?;

        Ok(self.size())
    }
}
