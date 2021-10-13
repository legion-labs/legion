use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{Error, FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

/// Sample Size Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct StszAtom {
    pub version: u8,
    pub flags: u32,
    pub sample_size: u32,
    pub sample_count: u32,

    #[serde(skip_serializing)]
    pub sample_sizes: Vec<u32>,
}

impl Atom for StszAtom {
    const FOUR_CC: FourCC = FourCC::new(b"stsz");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8 + (4 * self.sample_sizes.len() as u64)
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "sample_size={} sample_count={} sample_sizes={}",
            self.sample_size,
            self.sample_count,
            self.sample_sizes.len()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for StszAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let sample_size = reader.read_u32::<BigEndian>()?;
        let sample_count = reader.read_u32::<BigEndian>()?;
        let mut sample_sizes = Vec::with_capacity(sample_count as usize);
        if sample_size == 0 {
            for _ in 0..sample_count {
                let sample_number = reader.read_u32::<BigEndian>()?;
                sample_sizes.push(sample_number);
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            sample_size,
            sample_count,
            sample_sizes,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for StszAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.sample_size)?;
        writer.write_u32::<BigEndian>(self.sample_count)?;
        if self.sample_size == 0 {
            if self.sample_count != self.sample_sizes.len() as u32 {
                return Err(Error::InvalidData("sample count out of sync"));
            }
            for sample_number in &self.sample_sizes {
                writer.write_u32::<BigEndian>(*sample_number)?;
            }
        }

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_stsz_same_size() {
        let src_box = StszAtom {
            version: 0,
            flags: 0,
            sample_size: 1165,
            sample_count: 12,
            sample_sizes: vec![],
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, StszAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = StszAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_stsz_many_sizes() {
        let src_box = StszAtom {
            version: 0,
            flags: 0,
            sample_size: 0,
            sample_count: 9,
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, StszAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = StszAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
