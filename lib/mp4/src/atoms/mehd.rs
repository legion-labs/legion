use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MehdAtom {
    pub version: u8,
    pub flags: u32,
    pub fragment_duration: u64,
}

impl Default for MehdAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 0,
            fragment_duration: 0,
        }
    }
}

impl Atom for MehdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"mehd");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;

        if self.version == 1 {
            size += 8;
        } else if self.version == 0 {
            size += 4;
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("fragment_duration={}", self.fragment_duration);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MehdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let fragment_duration = if version == 1 {
            reader.read_u64::<BigEndian>()?
        } else if version == 0 {
            u64::from(reader.read_u32::<BigEndian>()?)
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        };
        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            fragment_duration,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for MehdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.fragment_duration)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.fragment_duration as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
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
    fn test_mehd32() {
        let src_box = MehdAtom {
            version: 0,
            flags: 0,
            fragment_duration: 32,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MehdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MehdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mehd64() {
        let src_box = MehdAtom {
            version: 0,
            flags: 0,
            fragment_duration: 30439936,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MehdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MehdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
