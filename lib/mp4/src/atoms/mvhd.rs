use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, value_u32, write_atom_header_ext, Atom, AtomHeader,
    FixedPointU16, FixedPointU8, Matrix, ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

/// Movie Header Atom
/// This box defines overall information which is media-independent, and relevant to the entire presentation considered as a whole
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MvhdAtom {
    pub version: u8,
    pub flags: u32,
    pub creation_time: u64,
    pub modification_time: u64,
    pub timescale: u32,
    pub duration: u64,

    #[serde(with = "value_u32")]
    pub rate: FixedPointU16,

    pub volume: FixedPointU8,
    pub matrix: Matrix,
    pub next_track_id: u32,
}

impl Default for MvhdAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 0,
            creation_time: 0,
            modification_time: 0,
            timescale: 1000,
            duration: 0,
            rate: FixedPointU16::new(1),
            volume: FixedPointU8::new(1),
            matrix: Matrix::default(),
            next_track_id: 0,
        }
    }
}

impl Atom for MvhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"mvhd");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;
        if self.version == 1 {
            size += 28;
        } else if self.version == 0 {
            size += 16;
        }
        size += 80;
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "creation_time={} timescale={} duration={} rate={}",
            self.creation_time,
            self.timescale,
            self.duration,
            self.rate.value()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MvhdAtom {
    fn read_atom(reader: &mut R, _size: u64) -> Result<Self> {
        box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let (creation_time, modification_time, timescale, duration) = if version == 1 {
            (
                reader.read_u64::<BigEndian>()?,
                reader.read_u64::<BigEndian>()?,
                reader.read_u32::<BigEndian>()?,
                reader.read_u64::<BigEndian>()?,
            )
        } else if version == 0 {
            (
                u64::from(reader.read_u32::<BigEndian>()?),
                u64::from(reader.read_u32::<BigEndian>()?),
                reader.read_u32::<BigEndian>()?,
                u64::from(reader.read_u32::<BigEndian>()?),
            )
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        };
        let rate = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);
        let volume = FixedPointU8::new_raw(reader.read_u16::<BigEndian>()?);

        reader.read_u64::<BigEndian>()?; // reserved
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

        //  pre_defined
        reader.read_u64::<BigEndian>()?;
        reader.read_u64::<BigEndian>()?;
        reader.read_u64::<BigEndian>()?;

        let next_track_id = reader.read_u32::<BigEndian>()?;

        Ok(Self {
            version,
            flags,
            creation_time,
            modification_time,
            timescale,
            duration,
            rate,
            volume,
            matrix,
            next_track_id,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for MvhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        if self.version == 1 {
            writer.write_u64::<BigEndian>(self.creation_time)?;
            writer.write_u64::<BigEndian>(self.modification_time)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u64::<BigEndian>(self.duration)?;
        } else if self.version == 0 {
            writer.write_u32::<BigEndian>(self.creation_time as u32)?;
            writer.write_u32::<BigEndian>(self.modification_time as u32)?;
            writer.write_u32::<BigEndian>(self.timescale)?;
            writer.write_u32::<BigEndian>(self.duration as u32)?;
        } else {
            return Err(Error::InvalidData("version must be 0 or 1"));
        }
        writer.write_u32::<BigEndian>(self.rate.raw_value())?;

        writer.write_u16::<BigEndian>(self.volume.raw_value())?;

        writer.write_u64::<BigEndian>(0)?; // reserved
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

        //  pre_defined
        writer.write_u64::<BigEndian>(0)?;
        writer.write_u64::<BigEndian>(0)?;
        writer.write_u64::<BigEndian>(0)?;

        writer.write_u32::<BigEndian>(self.next_track_id)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_mvhd32() {
        let src_box = MvhdAtom {
            version: 0,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: FixedPointU16::new(1),
            volume: FixedPointU8::new(1),
            matrix: Matrix::default(),
            next_track_id: 2,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MvhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MvhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mvhd64() {
        let src_box = MvhdAtom {
            version: 1,
            flags: 0,
            creation_time: 100,
            modification_time: 200,
            timescale: 1000,
            duration: 634634,
            rate: FixedPointU16::new(1),
            volume: FixedPointU8::new(1),
            matrix: Matrix::default(),
            next_track_id: 2,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MvhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MvhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
