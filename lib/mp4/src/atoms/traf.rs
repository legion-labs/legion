use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::tfhd::TfhdAtom;
use super::trun::TrunAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

/// Track Fragment Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrafAtom {
    pub tfhd: TfhdAtom,
    pub trun: Option<TrunAtom>,
}

impl Atom for TrafAtom {
    const FOUR_CC: FourCC = FourCC::new(b"traf");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tfhd.size();
        if let Some(ref trun) = self.trun {
            size += trun.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TrafAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tfhd = None;
        let mut trun = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                TfhdAtom::FOUR_CC => {
                    tfhd = Some(TfhdAtom::read_atom(reader, s)?);
                }
                TrunAtom::FOUR_CC => {
                    trun = Some(TrunAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if tfhd.is_none() {
            return Err(Error::BoxNotFound(TfhdAtom::FOUR_CC));
        }
        let tfhd = tfhd.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { tfhd, trun })
    }
}

impl<W: Write> WriteAtom<&mut W> for TrafAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.tfhd.write_atom(writer)?;

        Ok(self.size())
    }
}
