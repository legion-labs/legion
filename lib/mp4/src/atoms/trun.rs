use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{Error, FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrunAtom {
    pub version: u8,
    pub flags: u32,
    pub sample_count: u32,
    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<u32>,

    #[serde(skip_serializing)]
    pub sample_durations: Vec<u32>,
    #[serde(skip_serializing)]
    pub sample_sizes: Vec<u32>,
    #[serde(skip_serializing)]
    pub sample_flags: Vec<u32>,
    #[serde(skip_serializing)]
    pub sample_cts: Vec<u32>,
}

impl TrunAtom {
    pub const FLAG_DATA_OFFSET: u32 = 0x01;
    pub const FLAG_FIRST_SAMPLE_FLAGS: u32 = 0x04;
    pub const FLAG_SAMPLE_DURATION: u32 = 0x100;
    pub const FLAG_SAMPLE_SIZE: u32 = 0x200;
    pub const FLAG_SAMPLE_FLAGS: u32 = 0x400;
    pub const FLAG_SAMPLE_CTS: u32 = 0x800;
}

impl Atom for TrunAtom {
    const FOUR_CC: FourCC = FourCC::new(b"trun");

    fn size(&self) -> u64 {
        let mut sum = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if Self::FLAG_DATA_OFFSET & self.flags > 0 {
            sum += 4;
        }
        if Self::FLAG_FIRST_SAMPLE_FLAGS & self.flags > 0 {
            sum += 4;
        }
        if Self::FLAG_SAMPLE_DURATION & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        if Self::FLAG_SAMPLE_SIZE & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        if Self::FLAG_SAMPLE_FLAGS & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        if Self::FLAG_SAMPLE_CTS & self.flags > 0 {
            sum += 4 * self.sample_count as u64;
        }
        sum
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("sample_size={}", self.sample_count);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TrunAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let sample_count = reader.read_u32::<BigEndian>()?;

        let data_offset = if Self::FLAG_DATA_OFFSET & flags > 0 {
            Some(reader.read_i32::<BigEndian>()?)
        } else {
            None
        };

        let first_sample_flags = if Self::FLAG_FIRST_SAMPLE_FLAGS & flags > 0 {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };

        let mut sample_durations = Vec::with_capacity(sample_count as usize);
        let mut sample_sizes = Vec::with_capacity(sample_count as usize);
        let mut sample_flags = Vec::with_capacity(sample_count as usize);
        let mut sample_cts = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count {
            if Self::FLAG_SAMPLE_DURATION & flags > 0 {
                let duration = reader.read_u32::<BigEndian>()?;
                sample_durations.push(duration);
            }

            if Self::FLAG_SAMPLE_SIZE & flags > 0 {
                let sample_size = reader.read_u32::<BigEndian>()?;
                sample_sizes.push(sample_size);
            }

            if Self::FLAG_SAMPLE_FLAGS & flags > 0 {
                let sample_flag = reader.read_u32::<BigEndian>()?;
                sample_flags.push(sample_flag);
            }

            if Self::FLAG_SAMPLE_CTS & flags > 0 {
                let cts = reader.read_u32::<BigEndian>()?;
                sample_cts.push(cts);
            }
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            sample_count,
            data_offset,
            first_sample_flags,
            sample_durations,
            sample_sizes,
            sample_flags,
            sample_cts,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for TrunAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(self.sample_count)?;
        if let Some(v) = self.data_offset {
            writer.write_i32::<BigEndian>(v)?;
        }
        if let Some(v) = self.first_sample_flags {
            writer.write_u32::<BigEndian>(v)?;
        }
        if self.sample_count != self.sample_sizes.len() as u32 {
            return Err(Error::InvalidData("sample count out of sync"));
        }
        for i in 0..self.sample_count as usize {
            if Self::FLAG_SAMPLE_DURATION & self.flags > 0 {
                writer.write_u32::<BigEndian>(self.sample_durations[i])?;
            }
            if Self::FLAG_SAMPLE_SIZE & self.flags > 0 {
                writer.write_u32::<BigEndian>(self.sample_sizes[i])?;
            }
            if Self::FLAG_SAMPLE_FLAGS & self.flags > 0 {
                writer.write_u32::<BigEndian>(self.sample_flags[i])?;
            }
            if Self::FLAG_SAMPLE_CTS & self.flags > 0 {
                writer.write_u32::<BigEndian>(self.sample_cts[i])?;
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
    fn test_trun_same_size() {
        let src_box = TrunAtom {
            version: 0,
            flags: 0,
            data_offset: None,
            sample_count: 0,
            sample_sizes: vec![],
            sample_flags: vec![],
            first_sample_flags: None,
            sample_durations: vec![],
            sample_cts: vec![],
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TrunAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TrunAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_trun_many_sizes() {
        let src_box = TrunAtom {
            version: 0,
            flags: TrunAtom::FLAG_SAMPLE_DURATION
                | TrunAtom::FLAG_SAMPLE_SIZE
                | TrunAtom::FLAG_SAMPLE_FLAGS
                | TrunAtom::FLAG_SAMPLE_CTS,
            data_offset: None,
            sample_count: 9,
            sample_sizes: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
            sample_flags: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
            first_sample_flags: None,
            sample_durations: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
            sample_cts: vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730],
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TrunAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TrunAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
