use std::io::{Read, Seek, Write};

use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, value_u32, value_u8, write_atom_header_ext,
    Atom, AtomHeader, FixedPointU16, FixedPointU8, Matrix, ReadAtom, WriteAtom, HEADER_EXT_SIZE,
    HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

bitflags! {
    pub struct TrackFlags: u32 {
        const ENABLED = 0x000001;
        const IN_MOVIE = 0x000002;
        const IN_PREVIEW = 0x000004;
        const ALL = Self::ENABLED.bits | Self::IN_MOVIE.bits | Self::IN_PREVIEW.bits;
    }
}

/// Track Header Atom
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TkhdAtom {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub track_id: u32,
    pub duration: u64,
    pub layer: u16,
    pub alternate_group: u16,

    #[serde(with = "value_u8")]
    pub volume: FixedPointU8,
    pub matrix: Matrix,

    #[serde(with = "value_u32")]
    pub width: FixedPointU16,

    #[serde(with = "value_u32")]
    pub height: FixedPointU16,
}

impl Default for TkhdAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: TrackFlags::ALL.bits,
            creation_time: 0,
            modification_time: 0,
            track_id: 0,
            duration: 0,
            layer: 0,
            alternate_group: 0,
            volume: FixedPointU8::new(1),
            matrix: Matrix::default(),
            width: FixedPointU16::new(0),
            height: FixedPointU16::new(0),
        }
    }
}

impl TkhdAtom {
    pub fn set_width(&mut self, width: u16) {
        self.width = FixedPointU16::new(width);
    }

    pub fn set_height(&mut self, height: u16) {
        self.height = FixedPointU16::new(height);
    }
}

impl Atom for TkhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"tkhd");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 32;
        } else if self.version == 0 {
            size += 20;
        }
        size += 60;
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "creation_time={} track_id={} duration={} layer={} volume={} width={} height={}",
            self.creation_time,
            self.track_id,
            self.duration,
            self.layer,
            self.volume.value(),
            self.width.value(),
            self.height.value()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TkhdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let (creation_time, modification_time, track_id, _, duration) = if version == 1 {
            (
                reader.read_u64::<BigEndian>()?,
                reader.read_u64::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                reader.read_u64::<BigEndian>()?,
            )
        } else if version == 0 {
            (
                u64::from(reader.read_u32::<BigEndian>()?),
                u64::from(reader.read_u32::<BigEndian>()?),
                reader.read_u32::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                u64::from(reader.read_u32::<BigEndian>()?),
            )
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        };
        reader.read_u64::<BigEndian>()?; // reserved
        let layer = reader.read_u16::<BigEndian>()?;
        let alternate_group = reader.read_u16::<BigEndian>()?;
        let volume = FixedPointU8::new_raw(reader.read_u16::<BigEndian>()?);

        reader.read_u16::<BigEndian>()?; // reserved
        let matrix = Matrix {
            a: reader.read_i32::<BigEndian>()?,
            b: reader.read_i32::<BigEndian>()?,
            u: reader.read_i32::<BigEndian>()?,
            c: reader.read_i32::<BigEndian>()?,
            d: reader.read_i32::<BigEndian>()?,
            v: reader.read_i32::<BigEndian>()?,
            x: reader.read_i32::<BigEndian>()?,
            y: reader.read_i32::<BigEndian>()?,
            w: reader.read_i32::<BigEndian>()?,
        };

        let width = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        let height = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            creation_time,
            modification_time,
            track_id,
            duration,
            layer,
            alternate_group,
            volume,
            matrix,
            width,
            height,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for TkhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time)?;
            writer.write_u64::<BigEndian>(self.modification_time)?;
            writer.write_u32::<BigEndian>(self.track_id)?;
            writer.write_u32::<BigEndian>(0)?; // reserved
            writer.write_u64::<BigEndian>(self.duration)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.creation_time as u32)?;
            writer.write_u32::<BigEndian>(self.modification_time as u32)?;
            writer.write_u32::<BigEndian>(self.track_id)?;
            writer.write_u32::<BigEndian>(0)?; // reserved
            writer.write_u32::<BigEndian>(self.duration as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        }

        writer.write_u64::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.layer)?;
        writer.write_u16::<BigEndian>(self.alternate_group)?;
        writer.write_u16::<BigEndian>(self.volume.raw_value())?;

        writer.write_u16::<BigEndian>(0)?; // reserved

        writer.write_i32::<BigEndian>(self.matrix.a)?;
        writer.write_i32::<BigEndian>(self.matrix.b)?;
        writer.write_i32::<BigEndian>(self.matrix.u)?;
        writer.write_i32::<BigEndian>(self.matrix.c)?;
        writer.write_i32::<BigEndian>(self.matrix.d)?;
        writer.write_i32::<BigEndian>(self.matrix.v)?;
        writer.write_i32::<BigEndian>(self.matrix.x)?;
        writer.write_i32::<BigEndian>(self.matrix.y)?;
        writer.write_i32::<BigEndian>(self.matrix.w)?;

        writer.write_u32::<BigEndian>(self.width.raw_value())?;
        writer.write_u32::<BigEndian>(self.height.raw_value())?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_tkhd32() {
        let src_box = TkhdAtom {
            version: 0,
            flags: TrackFlags::ALL.bits,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: FixedPointU8::new(1),
            matrix: Matrix {
                a: 0x00010000,
                b: 0,
                u: 0,
                c: 0,
                d: 0x00010000,
                v: 0,
                x: 0,
                y: 0,
                w: 0x40000000,
            },
            width: FixedPointU16::new(512),
            height: FixedPointU16::new(288),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TkhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TkhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_tkhd64() {
        let src_box = TkhdAtom {
            version: 1,
            flags: TrackFlags::ALL.bits,
            creation_time: 100,
            modification_time: 200,
            track_id: 1,
            duration: 634634,
            layer: 0,
            alternate_group: 0,
            volume: FixedPointU8::new(1),
            matrix: Matrix {
                a: 0x00010000,
                b: 0,
                u: 0,
                c: 0,
                d: 0x00010000,
                v: 0,
                x: 0,
                y: 0,
                w: 0x40000000,
            },
            width: FixedPointU16::new(512),
            height: FixedPointU16::new(288),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TkhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TkhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
