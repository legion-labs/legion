use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use crate::{FourCC, Result, Vp9Config};

use super::vpcc::VpccAtom;
use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Vp09Atom {
    pub version: u8,
    pub flags: u32,
    pub start_code: u16,
    pub data_reference_index: u16,
    pub reserved0: [u8; 16],
    pub width: u16,
    pub height: u16,
    pub horizresolution: (u16, u16),
    pub vertresolution: (u16, u16),
    pub reserved1: [u8; 4],
    pub frame_count: u16,
    pub compressorname: [u8; 32],
    pub depth: u16,
    pub end_code: u16,
    pub vpcc: VpccAtom,
}

impl Vp09Atom {
    pub const DEFAULT_START_CODE: u16 = 0;
    pub const DEFAULT_END_CODE: u16 = 0xFFFF;
    pub const DEFAULT_DATA_REFERENCE_INDEX: u16 = 1;
    pub const DEFAULT_HORIZRESOLUTION: (u16, u16) = (0x48, 0x00);
    pub const DEFAULT_VERTRESOLUTION: (u16, u16) = (0x48, 0x00);
    pub const DEFAULT_FRAME_COUNT: u16 = 1;
    pub const DEFAULT_COMPRESSORNAME: [u8; 32] = [0; 32];
    pub const DEFAULT_DEPTH: u16 = 24;

    pub fn new(config: &Vp9Config) -> Self {
        Self {
            version: 0,
            flags: 0,
            start_code: Self::DEFAULT_START_CODE,
            data_reference_index: Self::DEFAULT_DATA_REFERENCE_INDEX,
            reserved0: Default::default(),
            width: config.width,
            height: config.height,
            horizresolution: Self::DEFAULT_HORIZRESOLUTION,
            vertresolution: Self::DEFAULT_VERTRESOLUTION,
            reserved1: Default::default(),
            frame_count: Self::DEFAULT_FRAME_COUNT,
            compressorname: Self::DEFAULT_COMPRESSORNAME,
            depth: Self::DEFAULT_DEPTH,
            end_code: Self::DEFAULT_END_CODE,
            vpcc: VpccAtom {
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
            },
        }
    }
}

impl Atom for Vp09Atom {
    const FOUR_CC: FourCC = FourCC::new(b"vp09");

    fn size(&self) -> u64 {
        0x6A
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        Ok(format!("{:?}", self))
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for Vp09Atom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;
        let (version, flags) = read_atom_header_ext(reader)?;

        let start_code: u16 = reader.read_u16::<BigEndian>()?;
        let data_reference_index: u16 = reader.read_u16::<BigEndian>()?;
        let reserved0: [u8; 16] = {
            let mut buf = [0u8; 16];
            reader.read_exact(&mut buf)?;
            buf
        };
        let width: u16 = reader.read_u16::<BigEndian>()?;
        let height: u16 = reader.read_u16::<BigEndian>()?;
        let horizresolution: (u16, u16) = (
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
        );
        let vertresolution: (u16, u16) = (
            reader.read_u16::<BigEndian>()?,
            reader.read_u16::<BigEndian>()?,
        );
        let reserved1: [u8; 4] = {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            buf
        };
        let frame_count: u16 = reader.read_u16::<BigEndian>()?;
        let compressorname: [u8; 32] = {
            let mut buf = [0u8; 32];
            reader.read_exact(&mut buf)?;
            buf
        };
        let depth: u16 = reader.read_u16::<BigEndian>()?;
        let end_code: u16 = reader.read_u16::<BigEndian>()?;

        let vpcc = {
            let header = AtomHeader::read(reader)?;
            VpccAtom::read_atom(reader, header.size)?
        };

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            start_code,
            data_reference_index,
            reserved0,
            width,
            height,
            horizresolution,
            vertresolution,
            reserved1,
            frame_count,
            compressorname,
            depth,
            end_code,
            vpcc,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for Vp09Atom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u16::<BigEndian>(self.start_code)?;
        writer.write_u16::<BigEndian>(self.data_reference_index)?;
        writer.write_all(&self.reserved0)?;
        writer.write_u16::<BigEndian>(self.width)?;
        writer.write_u16::<BigEndian>(self.height)?;
        writer.write_u16::<BigEndian>(self.horizresolution.0)?;
        writer.write_u16::<BigEndian>(self.horizresolution.1)?;
        writer.write_u16::<BigEndian>(self.vertresolution.0)?;
        writer.write_u16::<BigEndian>(self.vertresolution.1)?;
        writer.write_all(&self.reserved1)?;
        writer.write_u16::<BigEndian>(self.frame_count)?;
        writer.write_all(&self.compressorname)?;
        writer.write_u16::<BigEndian>(self.depth)?;
        writer.write_u16::<BigEndian>(self.end_code)?;
        VpccAtom::write_atom(&self.vpcc, writer)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_vpcc() {
        let src_box = Vp09Atom::new(&Vp9Config {
            width: 1920,
            height: 1080,
        });
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, Vp09Atom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = Vp09Atom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
