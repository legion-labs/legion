use std::io::{Read, Seek, SeekFrom, Write};

use serde::Serialize;

use super::co64::Co64Atom;
use super::ctts::CttsAtom;
use super::stco::StcoAtom;
use super::stsc::StscAtom;
use super::stsd::StsdAtom;
use super::stss::StssAtom;
use super::stsz::StszAtom;
use super::stts::SttsAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

/// Sample Table Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct StblAtom {
    pub stsd: StsdAtom,
    pub stts: SttsAtom,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctts: Option<CttsAtom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stss: Option<StssAtom>,
    pub stsc: StscAtom,
    pub stsz: StszAtom,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stco: Option<StcoAtom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub co64: Option<Co64Atom>,
}

impl Atom for StblAtom {
    const FOUR_CC: FourCC = FourCC::new(b"stbl");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.stsd.size();
        size += self.stts.size();
        if let Some(ref ctts) = self.ctts {
            size += ctts.size();
        }
        if let Some(ref stss) = self.stss {
            size += stss.size();
        }
        size += self.stsc.size();
        size += self.stsz.size();
        if let Some(ref stco) = self.stco {
            size += stco.size();
        }
        if let Some(ref co64) = self.co64 {
            size += co64.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for StblAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut stsd = None;
        let mut stts = None;
        let mut ctts = None;
        let mut stss = None;
        let mut stsc = None;
        let mut stsz = None;
        let mut stco = None;
        let mut co64 = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                StsdAtom::FOUR_CC => {
                    stsd = Some(StsdAtom::read_atom(reader, s)?);
                }
                SttsAtom::FOUR_CC => {
                    stts = Some(SttsAtom::read_atom(reader, s)?);
                }
                CttsAtom::FOUR_CC => {
                    ctts = Some(CttsAtom::read_atom(reader, s)?);
                }
                StssAtom::FOUR_CC => {
                    stss = Some(StssAtom::read_atom(reader, s)?);
                }
                StscAtom::FOUR_CC => {
                    stsc = Some(StscAtom::read_atom(reader, s)?);
                }
                StszAtom::FOUR_CC => {
                    stsz = Some(StszAtom::read_atom(reader, s)?);
                }
                StcoAtom::FOUR_CC => {
                    stco = Some(StcoAtom::read_atom(reader, s)?);
                }
                Co64Atom::FOUR_CC => {
                    co64 = Some(Co64Atom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if stsd.is_none() {
            return Err(Error::BoxNotFound(StsdAtom::FOUR_CC));
        }
        let stsd = stsd.unwrap();
        if stts.is_none() {
            return Err(Error::BoxNotFound(SttsAtom::FOUR_CC));
        }
        let stts = stts.unwrap();
        if stsc.is_none() {
            return Err(Error::BoxNotFound(StscAtom::FOUR_CC));
        }
        let stsc = stsc.unwrap();
        if stsz.is_none() {
            return Err(Error::BoxNotFound(StszAtom::FOUR_CC));
        }
        let stsz = stsz.unwrap();
        if stco.is_none() && co64.is_none() {
            return Err(Error::Box2NotFound(StcoAtom::FOUR_CC, Co64Atom::FOUR_CC));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            stsd,
            stts,
            ctts,
            stss,
            stsc,
            stsz,
            stco,
            co64,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for StblAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.stsd.write_atom(writer)?;
        self.stts.write_atom(writer)?;
        if let Some(ref ctts) = self.ctts {
            ctts.write_atom(writer)?;
        }
        if let Some(ref stss) = self.stss {
            stss.write_atom(writer)?;
        }
        self.stsc.write_atom(writer)?;
        self.stsz.write_atom(writer)?;
        if let Some(ref stco) = self.stco {
            stco.write_atom(writer)?;
        }
        if let Some(ref co64) = self.co64 {
            co64.write_atom(writer)?;
        }

        Ok(self.size())
    }
}
