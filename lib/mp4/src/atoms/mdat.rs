use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{FourCC, Result};

use super::{box_start, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum MdatAtom<'a> {
    Owned(Vec<u8>),
    Borrowed(&'a [u8]),
}

impl<'a> MdatAtom<'a> {
    fn data_size(&self) -> u64 {
        match self {
            MdatAtom::Owned(data) => data.len() as u64,
            MdatAtom::Borrowed(data) => data.len() as u64,
        }
    }

    fn data(&self) -> &[u8] {
        match self {
            MdatAtom::Owned(data) => data,
            MdatAtom::Borrowed(data) => data,
        }
    }
}

impl<'a> Atom for MdatAtom<'a> {
    const FOUR_CC: FourCC = FourCC::new(b"mdat");

    fn size(&self) -> u64 {
        HEADER_SIZE + self.data_size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("data={}", self.data_size());
        Ok(s)
    }
}

impl<'a, R: Read + Seek> ReadAtom<&mut R> for MdatAtom<'a> {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        box_start(reader)?;
        let mut data = vec![0u8; size as usize - 8];
        reader.read_exact(&mut data)?;

        Ok(Self::Owned(data))
    }
}

impl<'a, W: Write> WriteAtom<&mut W> for MdatAtom<'a> {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        writer.write_all(self.data())?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_mdat_owned() {
        let src_box = MdatAtom::Owned(vec![100u8, 101u8, 102u8, 103u8, 103u8]);
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MdatAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MdatAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mdat_borrowed() {
        let data = vec![100u8, 101u8, 102u8, 103u8, 103u8];
        let src_box = MdatAtom::Borrowed(&data);
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MdatAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MdatAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box.data(), dst_box.data());
    }
}
