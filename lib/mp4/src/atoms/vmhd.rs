use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct VmhdAtom {
    pub version: u8,
    pub flags: u32,
    pub graphics_mode: u16,
    pub op_color: RgbColor,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct RgbColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl Atom for VmhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"vmhd");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 8
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "graphics_mode={} op_color={}{}{}",
            self.graphics_mode, self.op_color.red, self.op_color.green, self.op_color.blue
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for VmhdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let graphics_mode = reader.read_u16::<BigEndian>()?;
        let op_color = RgbColor {
            red: reader.read_u16::<BigEndian>()?,
            green: reader.read_u16::<BigEndian>()?,
            blue: reader.read_u16::<BigEndian>()?,
        };

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            graphics_mode,
            op_color,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for VmhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u16::<BigEndian>(self.graphics_mode)?;
        writer.write_u16::<BigEndian>(self.op_color.red)?;
        writer.write_u16::<BigEndian>(self.op_color.green)?;
        writer.write_u16::<BigEndian>(self.op_color.blue)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::AtomHeader;
    use std::io::Cursor;

    #[test]
    fn test_vmhd() {
        let src_box = VmhdAtom {
            version: 0,
            flags: 1,
            graphics_mode: 0,
            op_color: RgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, VmhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = VmhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
