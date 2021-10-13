use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::dinf::DinfAtom;
use super::smhd::SmhdAtom;
use super::stbl::StblAtom;
use super::vmhd::VmhdAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

/// Media Information Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MinfAtom {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vmhd: Option<VmhdAtom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub smhd: Option<SmhdAtom>,

    pub dinf: DinfAtom,
    pub stbl: StblAtom,
}

impl Atom for MinfAtom {
    const FOUR_CC: FourCC = FourCC::new(b"minf");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref vmhd) = self.vmhd {
            size += vmhd.size();
        }
        if let Some(ref smhd) = self.smhd {
            size += smhd.size();
        }
        size += self.dinf.size();
        size += self.stbl.size();
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

impl<R: Read + Seek> ReadAtom<&mut R> for MinfAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut vmhd = None;
        let mut smhd = None;
        let mut dinf = None;
        let mut stbl = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                VmhdAtom::FOUR_CC => {
                    vmhd = Some(VmhdAtom::read_atom(reader, s)?);
                }
                SmhdAtom::FOUR_CC => {
                    smhd = Some(SmhdAtom::read_atom(reader, s)?);
                }
                DinfAtom::FOUR_CC => {
                    dinf = Some(DinfAtom::read_atom(reader, s)?);
                }
                StblAtom::FOUR_CC => {
                    stbl = Some(StblAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if dinf.is_none() {
            return Err(Error::BoxNotFound(DinfAtom::FOUR_CC));
        }
        let dinf = dinf.unwrap();
        if stbl.is_none() {
            return Err(Error::BoxNotFound(StblAtom::FOUR_CC));
        }
        let stbl = stbl.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            vmhd,
            smhd,
            dinf,
            stbl,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for MinfAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        if let Some(ref vmhd) = self.vmhd {
            vmhd.write_atom(writer)?;
        }
        if let Some(ref smhd) = self.smhd {
            smhd.write_atom(writer)?;
        }
        self.dinf.write_atom(writer)?;
        self.stbl.write_atom(writer)?;

        Ok(self.size())
    }
}
