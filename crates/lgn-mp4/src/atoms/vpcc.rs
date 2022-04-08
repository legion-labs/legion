use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{FourCC, Result};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct VpccAtom {
    pub version: u8,
    pub flags: u32,
    pub profile: u8,
    pub level: u8,
    pub bit_depth: u8,
    pub chroma_subsampling: u8,
    pub video_full_range_flag: bool,
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
    pub codec_initialization_data_size: u16,
}

impl VpccAtom {
    pub const DEFAULT_VERSION: u8 = 1;
    pub const DEFAULT_BIT_DEPTH: u8 = 8;
}

impl Atom for VpccAtom {
    const FOUR_CC: FourCC = FourCC::new(b"vpcc");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("{:?}", self))
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for VpccAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;
        let (version, flags) = read_atom_header_ext(reader)?;

        let profile: u8 = reader.read_u8()?;
        let level: u8 = reader.read_u8()?;
        let (bit_depth, chroma_subsampling, video_full_range_flag) = {
            let b = reader.read_u8()?;
            (b >> 4, b << 4 >> 5, b & 0x01 == 1)
        };
        let transfer_characteristics: u8 = reader.read_u8()?;
        let matrix_coefficients: u8 = reader.read_u8()?;
        let codec_initialization_data_size: u16 = reader.read_u16::<BigEndian>()?;

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            profile,
            level,
            bit_depth,
            chroma_subsampling,
            video_full_range_flag,
            color_primaries: 0,
            transfer_characteristics,
            matrix_coefficients,
            codec_initialization_data_size,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for VpccAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u8(self.profile)?;
        writer.write_u8(self.level)?;
        writer.write_u8(
            (self.bit_depth << 4)
                | (self.chroma_subsampling << 1)
                | u8::from(self.video_full_range_flag),
        )?;
        writer.write_u8(self.color_primaries)?;
        writer.write_u8(self.transfer_characteristics)?;
        writer.write_u8(self.matrix_coefficients)?;
        writer.write_u16::<BigEndian>(self.codec_initialization_data_size)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_vpcc() {
        let src_box = VpccAtom {
            version: VpccAtom::DEFAULT_VERSION,
            flags: 0,
            profile: 0,
            level: 0x1F,
            bit_depth: VpccAtom::DEFAULT_BIT_DEPTH,
            chroma_subsampling: 0,
            video_full_range_flag: false,
            color_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            codec_initialization_data_size: 0,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, VpccAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = VpccAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
