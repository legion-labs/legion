use std::io::{Read, Seek, SeekFrom, Write};

use serde::Serialize;

use super::dref::DrefAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

/// Data Information Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DinfAtom {
    dref: DrefAtom,
}

impl Atom for DinfAtom {
    const FOUR_CC: FourCC = FourCC::new(b"dinf");

    fn size(&self) -> u64 {
        HEADER_SIZE + self.dref.size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for DinfAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut dref = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                DrefAtom::FOUR_CC => {
                    dref = Some(DrefAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if dref.is_none() {
            return Err(Error::BoxNotFound(DrefAtom::FOUR_CC));
        }
        let dref = dref.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { dref })
    }
}

impl<W: Write> WriteAtom<&mut W> for DinfAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;
        self.dref.write_atom(writer)?;
        Ok(self.size())
    }
}
