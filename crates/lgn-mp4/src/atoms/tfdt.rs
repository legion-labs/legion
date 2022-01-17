use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, write_atom_header_ext, Atom, AtomHeader, ReadAtom, WriteAtom,
    HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct TfdtAtom {
    pub version: u8,
    pub flags: u32,

    pub decode_time: u64,
}

impl Atom for TfdtAtom {
    const FOUR_CC: FourCC = FourCC::new(b"tfdt");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + if self.version == 0 { 4 } else { 8 }
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("decode_time={}", self.decode_time);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TfdtAtom {
    fn read_atom(reader: &mut R, _size: u64) -> Result<Self> {
        box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;
        let decode_time = if version == 0 {
            u64::from(reader.read_u32::<BigEndian>()?)
        } else if version == 1 {
            reader.read_u64::<BigEndian>()?
        } else {
            return Err(Error::InvalidData("tfdt version not supported"));
        };

        Ok(Self {
            version,
            flags,
            decode_time,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for TfdtAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;
        if self.version == 0 {
            writer.write_u32::<BigEndian>(
                self.decode_time
                    .try_into()
                    .map_err(|_err| Error::InvalidData("decode time too small, use version 1"))?,
            )?;
        } else if self.version == 1 {
            writer.write_u64::<BigEndian>(self.decode_time)?;
        } else {
            return Err(Error::InvalidData("tdft version not supported"));
        }

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_smhd_v0() {
        let src_box = TfdtAtom {
            version: 0,
            flags: 0,
            decode_time: 0x000001000,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TfdtAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TfdtAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_smhd_v1() {
        let src_box = TfdtAtom {
            version: 1,
            flags: 0,
            decode_time: 0x000001000,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TfdtAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TfdtAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
