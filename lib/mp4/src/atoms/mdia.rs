use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::hdlr::HdlrAtom;
use super::mdhd::MdhdAtom;
use super::minf::MinfAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

/// Media Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MdiaAtom {
    pub mdhd: MdhdAtom,
    pub hdlr: HdlrAtom,
    pub minf: MinfAtom,
}

impl Atom for MdiaAtom {
    const FOUR_CC: FourCC = FourCC::new(b"mdia");

    fn size(&self) -> u64 {
        HEADER_SIZE + self.mdhd.size() + self.hdlr.size() + self.minf.size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MdiaAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mdhd = None;
        let mut hdlr = None;
        let mut minf = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                MdhdAtom::FOUR_CC => {
                    mdhd = Some(MdhdAtom::read_atom(reader, s)?);
                }
                HdlrAtom::FOUR_CC => {
                    hdlr = Some(HdlrAtom::read_atom(reader, s)?);
                }
                MinfAtom::FOUR_CC => {
                    minf = Some(MinfAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mdhd.is_none() {
            return Err(Error::BoxNotFound(MdhdAtom::FOUR_CC));
        }
        let mdhd = mdhd.unwrap();
        if hdlr.is_none() {
            return Err(Error::BoxNotFound(HdlrAtom::FOUR_CC));
        }
        let hdlr = hdlr.unwrap();
        if minf.is_none() {
            return Err(Error::BoxNotFound(MinfAtom::FOUR_CC));
        }
        let minf = minf.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { mdhd, hdlr, minf })
    }
}

impl<W: Write> WriteAtom<&mut W> for MdiaAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.mdhd.write_atom(writer)?;
        self.hdlr.write_atom(writer)?;
        self.minf.write_atom(writer)?;

        Ok(self.size())
    }
}
