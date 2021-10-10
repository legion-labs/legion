use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::mfhd::MfhdAtom;
use super::traf::TrafAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MoofAtom {
    pub mfhd: MfhdAtom,

    #[serde(rename = "traf")]
    pub trafs: Vec<TrafAtom>,
}

impl Atom for MoofAtom {
    const FOUR_CC: FourCC = FourCC::new(b"moof");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mfhd.size();
        for traf in &self.trafs {
            size += traf.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("trafs={}", self.trafs.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MoofAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mfhd = None;
        let mut trafs = Vec::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                MfhdAtom::FOUR_CC => {
                    mfhd = Some(MfhdAtom::read_atom(reader, s)?);
                }
                TrafAtom::FOUR_CC => {
                    let traf = TrafAtom::read_atom(reader, s)?;
                    trafs.push(traf);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mfhd.is_none() {
            return Err(Error::BoxNotFound(MfhdAtom::FOUR_CC));
        }
        let mfhd = mfhd.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { mfhd, trafs })
    }
}

impl<W: Write> WriteAtom<&mut W> for MoofAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.mfhd.write_atom(writer)?;
        for traf in &self.trafs {
            traf.write_atom(writer)?;
        }
        Ok(0)
    }
}
