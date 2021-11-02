use std::io::{Read, Seek, Write};

use serde::Serialize;

use super::elst::ElstAtom;
use super::{box_start, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE};
use crate::{FourCC, Result};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct EdtsAtom {
    pub elst: Option<ElstAtom>,
}

impl Atom for EdtsAtom {
    const FOUR_CC: FourCC = FourCC::new(b"edts");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        if let Some(ref elst) = self.elst {
            size += elst.size();
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

impl<R: Read + Seek> ReadAtom<&mut R> for EdtsAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut edts = Self::default();

        let header = AtomHeader::read(reader)?;
        let AtomHeader { name, size: s } = header;

        if name == ElstAtom::FOUR_CC {
            let elst = ElstAtom::read_atom(reader, s)?;
            edts.elst = Some(elst);
        }

        skip_bytes_to(reader, start + size)?;

        Ok(edts)
    }
}

impl<W: Write> WriteAtom<&mut W> for EdtsAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        if let Some(ref elst) = self.elst {
            elst.write_atom(writer)?;
        }

        Ok(self.size())
    }
}
