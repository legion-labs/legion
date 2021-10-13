use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::edts::EdtsAtom;
use super::mdia::MdiaAtom;
use super::tkhd::TkhdAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

/// Track Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrakAtom {
    pub tkhd: TkhdAtom,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub edts: Option<EdtsAtom>,

    pub mdia: MdiaAtom,
}

impl Atom for TrakAtom {
    const FOUR_CC: FourCC = FourCC::new(b"trak");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tkhd.size();
        if let Some(ref edts) = self.edts {
            size += edts.size();
        }
        size += self.mdia.size();
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

impl<R: Read + Seek> ReadAtom<&mut R> for TrakAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tkhd = None;
        let mut edts = None;
        let mut mdia = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                TkhdAtom::FOUR_CC => {
                    tkhd = Some(TkhdAtom::read_atom(reader, s)?);
                }
                EdtsAtom::FOUR_CC => {
                    edts = Some(EdtsAtom::read_atom(reader, s)?);
                }
                MdiaAtom::FOUR_CC => {
                    mdia = Some(MdiaAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if tkhd.is_none() {
            return Err(Error::BoxNotFound(TkhdAtom::FOUR_CC));
        }
        let tkhd = tkhd.unwrap();
        if mdia.is_none() {
            return Err(Error::BoxNotFound(MdiaAtom::FOUR_CC));
        }
        let mdia = mdia.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { tkhd, edts, mdia })
    }
}

impl<W: Write> WriteAtom<&mut W> for TrakAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.tkhd.write_atom(writer)?;
        if let Some(ref edts) = self.edts {
            edts.write_atom(writer)?;
        }
        self.mdia.write_atom(writer)?;

        Ok(self.size())
    }
}
