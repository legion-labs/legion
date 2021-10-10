use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::mvex::MvexAtom;
use super::mvhd::MvhdAtom;
use super::trak::TrakAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MoovAtom {
    pub mvhd: MvhdAtom,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mvex: Option<MvexAtom>,

    #[serde(rename = "trak")]
    pub traks: Vec<TrakAtom>,
}

impl Atom for MoovAtom {
    const FOUR_CC: FourCC = FourCC::new(b"moov");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mvhd.size();
        for trak in &self.traks {
            size += trak.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("traks={}", self.traks.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MoovAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mvhd = None;
        let mut mvex = None;
        let mut traks = Vec::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                MvhdAtom::FOUR_CC => {
                    mvhd = Some(MvhdAtom::read_atom(reader, s)?);
                }
                MvexAtom::FOUR_CC => {
                    mvex = Some(MvexAtom::read_atom(reader, s)?);
                }
                TrakAtom::FOUR_CC => {
                    let trak = TrakAtom::read_atom(reader, s)?;
                    traks.push(trak);
                }
                //UdtaBox::FOUR_CC => {
                //    // XXX warn!()
                //    skip_box(reader, s)?;
                //}
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mvhd.is_none() {
            return Err(Error::BoxNotFound(MvhdAtom::FOUR_CC));
        }
        let mvhd = mvhd.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { mvhd, mvex, traks })
    }
}

impl<W: Write> WriteAtom<&mut W> for MoovAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.mvhd.write_atom(writer)?;
        for trak in &self.traks {
            trak.write_atom(writer)?;
        }
        Ok(0)
    }
}
