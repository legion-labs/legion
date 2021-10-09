use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TfhdAtom {
    pub version: u8,
    pub flags: u32,
    pub track_id: u32,
    pub base_data_offset: u64,
}

impl Default for TfhdAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 0,
            track_id: 0,
            base_data_offset: 0,
        }
    }
}

impl Atom for TfhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"tfhd");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 + 8
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("track_id={}", self.track_id);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TfhdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;
        let track_id = reader.read_u32::<BigEndian>()?;
        let base_data_offset = reader.read_u64::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            track_id,
            base_data_offset,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for TfhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;
        writer.write_u32::<BigEndian>(self.track_id)?;
        writer.write_u64::<BigEndian>(self.base_data_offset)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_tfhd() {
        let src_box = TfhdAtom {
            version: 0,
            flags: 0,
            track_id: 1,
            base_data_offset: 0,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TfhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TfhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
