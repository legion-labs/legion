use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::avc1::Avc1Atom;
use super::hev1::Hev1Atom;
use super::mp4a::Mp4aAtom;
use super::tx3g::Tx3gAtom;
use super::vp09::Vp09Atom;
use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{FourCC, Result};

/// Sample Description Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct StsdAtom {
    pub version: u8,
    pub flags: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub avc1: Option<Avc1Atom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hev1: Option<Hev1Atom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub vp09: Option<Vp09Atom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mp4a: Option<Mp4aAtom>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx3g: Option<Tx3gAtom>,
}

impl Atom for StsdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"stsd");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if let Some(ref avc1) = self.avc1 {
            size += avc1.size();
        } else if let Some(ref hev1) = self.hev1 {
            size += hev1.size();
        } else if let Some(ref vp09) = self.vp09 {
            size += vp09.size();
        } else if let Some(ref mp4a) = self.mp4a {
            size += mp4a.size();
        } else if let Some(ref tx3g) = self.tx3g {
            size += tx3g.size();
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

impl<R: Read + Seek> ReadAtom<&mut R> for StsdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // XXX entry_count

        let mut avc1 = None;
        let mut hev1 = None;
        let mut vp09 = None;
        let mut mp4a = None;
        let mut tx3g = None;

        // Get box header.
        let header = AtomHeader::read(reader)?;
        let AtomHeader { name, size: s } = header;

        match name {
            Avc1Atom::FOUR_CC => {
                avc1 = Some(Avc1Atom::read_atom(reader, s)?);
            }
            Hev1Atom::FOUR_CC => {
                hev1 = Some(Hev1Atom::read_atom(reader, s)?);
            }
            Vp09Atom::FOUR_CC => {
                vp09 = Some(Vp09Atom::read_atom(reader, s)?);
            }
            Mp4aAtom::FOUR_CC => {
                mp4a = Some(Mp4aAtom::read_atom(reader, s)?);
            }
            Tx3gAtom::FOUR_CC => {
                tx3g = Some(Tx3gAtom::read_atom(reader, s)?);
            }
            _ => {}
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            avc1,
            hev1,
            vp09,
            mp4a,
            tx3g,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for StsdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(1)?; // entry_count

        if let Some(ref avc1) = self.avc1 {
            avc1.write_atom(writer)?;
        } else if let Some(ref hev1) = self.hev1 {
            hev1.write_atom(writer)?;
        } else if let Some(ref vp09) = self.vp09 {
            vp09.write_atom(writer)?;
        } else if let Some(ref mp4a) = self.mp4a {
            mp4a.write_atom(writer)?;
        } else if let Some(ref tx3g) = self.tx3g {
            tx3g.write_atom(writer)?;
        }

        Ok(self.size())
    }
}
