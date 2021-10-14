use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, SampleFlags, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

/// Track Extends Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrexAtom {
    pub version: u8,
    pub flags: u32,
    pub track_id: u32,
    pub default_sample_description_index: u32,
    pub default_sample_duration: u32,
    pub default_sample_size: u32,
    pub default_sample_flags: SampleFlags,
}

impl Atom for TrexAtom {
    const FOUR_CC: FourCC = FourCC::new(b"trex");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 20
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "track_id={} default_sample_duration={}",
            self.track_id, self.default_sample_duration
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TrexAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let track_id = reader.read_u32::<BigEndian>()?;
        let default_sample_description_index = reader.read_u32::<BigEndian>()?;
        let default_sample_duration = reader.read_u32::<BigEndian>()?;
        let default_sample_size = reader.read_u32::<BigEndian>()?;
        let default_sample_flags = reader.read_u32::<BigEndian>()?.into();

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            track_id,
            default_sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for TrexAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.track_id)?;
        writer.write_u32::<BigEndian>(self.default_sample_description_index)?;
        writer.write_u32::<BigEndian>(self.default_sample_duration)?;
        writer.write_u32::<BigEndian>(self.default_sample_size)?;
        writer.write_u32::<BigEndian>(self.default_sample_flags.into())?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_trex() {
        let src_box = TrexAtom {
            version: 0,
            flags: 0,
            track_id: 1,
            default_sample_description_index: 1,
            default_sample_duration: 1000,
            default_sample_size: 0,
            default_sample_flags: 65536.into(),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TrexAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TrexAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
