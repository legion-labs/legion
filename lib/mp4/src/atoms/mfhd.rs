use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

/// Movie Fragment Header Atom
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MfhdAtom {
    pub version: u8,
    pub flags: u32,
    pub sequence_number: u32,
}

impl Default for MfhdAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 0,
            sequence_number: 1,
        }
    }
}

impl Atom for MfhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"mfhd");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("sequence_number={}", self.sequence_number);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MfhdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;
        let sequence_number = reader.read_u32::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            sequence_number,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for MfhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;
        writer.write_u32::<BigEndian>(self.sequence_number)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_mfhd() {
        let src_box = MfhdAtom {
            version: 0,
            flags: 0,
            sequence_number: 1,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MfhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MfhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
