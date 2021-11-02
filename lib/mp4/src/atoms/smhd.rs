use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, value_i16, write_atom_header_ext, Atom,
    AtomHeader, FixedPointI8, ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{FourCC, Result};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SmhdAtom {
    pub version: u8,
    pub flags: u32,

    #[serde(with = "value_i16")]
    pub balance: FixedPointI8,
}

impl Default for SmhdAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 0,
            balance: FixedPointI8::new_raw(0),
        }
    }
}

impl Atom for SmhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"smhd");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("balance={}", self.balance.value());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for SmhdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let balance = FixedPointI8::new_raw(reader.read_i16::<BigEndian>()?);

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            balance,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for SmhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_i16::<BigEndian>(self.balance.raw_value())?;
        writer.write_u16::<BigEndian>(0)?; // reserved

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_smhd() {
        let src_box = SmhdAtom {
            version: 0,
            flags: 0,
            balance: FixedPointI8::new_raw(-1),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, SmhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = SmhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
