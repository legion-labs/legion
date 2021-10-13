use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{Error, FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

/// Track Fragment Run Atom
/// If the duration-is-empty flag is set in the `tf_flags`, there are no track runs.
/// A track run documents a contiguous set of samples for a track.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrunAtom {
    pub version: u8,
    pub sample_count: u32,
    pub data_offset: Option<i32>,
    pub first_sample_flags: Option<u32>,

    #[serde(skip_serializing)]
    pub sample_durations: Option<Vec<u32>>,
    #[serde(skip_serializing)]
    pub sample_sizes: Option<Vec<u32>>,
    #[serde(skip_serializing)]
    pub sample_flags: Option<Vec<u32>>,
    #[serde(skip_serializing)]
    pub sample_cts: Option<Vec<u32>>,
}

bitflags! {
    #[derive(Default, Serialize)]
    struct TrackRunlags: u32 {
        const DATA_OFFSET_PRESENT = 0x000001;
        /// this over-rides the default flags for the first sample only. This makes it possible
        /// to record a group of frames where the first is a key and the rest are difference frames,
        /// without supplying explicit flags for every sample. If this flag and field are used,
        /// sample-flags shall not be present.
        const FIRST_SAMPLE_FLAGS_PRESENT = 0x000004;
        /// indicates that each sample has its own duration, otherwise the default is used.
        const SAMPLE_DURATION_PRESENT = 0x000100;
        /// each sample has its own size, otherwise the default is used.
        const SAMPLE_SIZE_PRESENT = 0x000200;
        /// each sample has its own flags, otherwise the default is used.
        const SAMPLE_FLAGS_PRESENT = 0x000400;
        /// each sample has a composition time offset (e.g. as used for I/P/B video in MPEG).
        const SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT = 0x000800;
    }
}

impl From<&TrunAtom> for TrackRunlags {
    fn from(value: &TrunAtom) -> Self {
        let mut flags = Self::empty();
        flags.set(Self::DATA_OFFSET_PRESENT, value.data_offset.is_some());
        flags.set(
            Self::FIRST_SAMPLE_FLAGS_PRESENT,
            value.first_sample_flags.is_some(),
        );
        flags.set(
            Self::SAMPLE_DURATION_PRESENT,
            value.sample_durations.is_some(),
        );
        flags.set(Self::SAMPLE_SIZE_PRESENT, value.sample_sizes.is_some());
        flags.set(Self::SAMPLE_FLAGS_PRESENT, value.sample_flags.is_some());
        flags.set(
            Self::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT,
            value.sample_cts.is_some(),
        );
        flags
    }
}

impl Atom for TrunAtom {
    const FOUR_CC: FourCC = FourCC::new(b"trun");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 4 // sample_count
        + self.data_offset.map_or(0, |_| 4)
        + self.first_sample_flags.map_or(0, |_| 4)
        + self.sample_durations.as_ref().map_or(0, |_| 4 * u64::from(self.sample_count))
        + self.sample_sizes.as_ref().map_or(0, |_| 4 * u64::from(self.sample_count))
        + self.sample_flags.as_ref().map_or(0, |_| 4 * u64::from(self.sample_count))
        + self.sample_cts.as_ref().map_or(0, |_| 4 * u64::from(self.sample_count))
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
        let flags = TrackRunlags::from_bits(flags).ok_or(Error::InvalidData("invalid tr_flags"))?;
        let sample_count = reader.read_u32::<BigEndian>()?;

        let data_offset = if flags.contains(TrackRunlags::DATA_OFFSET_PRESENT) {
            Some(reader.read_i32::<BigEndian>()?)
        } else {
            None
        };

        let first_sample_flags = if flags.contains(TrackRunlags::FIRST_SAMPLE_FLAGS_PRESENT) {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };
        let mut sample_durations = Vec::with_capacity(sample_count as usize);
        let mut sample_sizes = Vec::with_capacity(sample_count as usize);
        let mut sample_flags = Vec::with_capacity(sample_count as usize);
        let mut sample_cts = Vec::with_capacity(sample_count as usize);
        for _ in 0..sample_count {
            if flags.contains(TrackRunlags::SAMPLE_DURATION_PRESENT) {
                let duration = reader.read_u32::<BigEndian>()?;
                sample_durations.push(duration);
            }

            if flags.contains(TrackRunlags::SAMPLE_SIZE_PRESENT) {
                let sample_size = reader.read_u32::<BigEndian>()?;
                sample_sizes.push(sample_size);
            }

            if flags.contains(TrackRunlags::SAMPLE_FLAGS_PRESENT) {
                let sample_flag = reader.read_u32::<BigEndian>()?;
                sample_flags.push(sample_flag);
            }

            if flags.contains(TrackRunlags::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT) {
                let cts = reader.read_u32::<BigEndian>()?;
                sample_cts.push(cts);
            }
        }
        let sample_durations = if flags.contains(TrackRunlags::SAMPLE_DURATION_PRESENT) {
            Some(sample_durations)
        } else {
            None
        };
        let sample_sizes = if flags.contains(TrackRunlags::SAMPLE_SIZE_PRESENT) {
            Some(sample_sizes)
        } else {
            None
        };
        let sample_flags = if flags.contains(TrackRunlags::SAMPLE_FLAGS_PRESENT) {
            Some(sample_flags)
        } else {
            None
        };
        let sample_cts = if flags.contains(TrackRunlags::SAMPLE_COMPOSITION_TIME_OFFSETS_PRESENT) {
            Some(sample_cts)
        } else {
            None
        };
        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
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

        let flags = TrackRunlags::from(self);
        write_atom_header_ext(writer, self.version, flags.bits())?;

        writer.write_u32::<BigEndian>(self.sample_count)?;
        if let Some(v) = self.data_offset {
            writer.write_i32::<BigEndian>(v)?;
        }
        if let Some(v) = self.first_sample_flags {
            writer.write_u32::<BigEndian>(v)?;
        }
        let sample_durations: &[u32] = self.sample_durations.as_ref().map_or(&[], |v| &v[..]);
        let sample_sizes: &[u32] = self.sample_sizes.as_ref().map_or(&[], |v| &v[..]);
        let sample_flags: &[u32] = self.sample_flags.as_ref().map_or(&[], |v| &v[..]);
        let sample_cts: &[u32] = self.sample_cts.as_ref().map_or(&[], |v| &v[..]);
        for i in 0..self.sample_count as usize {
            writer.write_u32::<BigEndian>(sample_durations[i])?;
            writer.write_u32::<BigEndian>(sample_sizes[i])?;
            writer.write_u32::<BigEndian>(sample_flags[i])?;
            writer.write_u32::<BigEndian>(sample_cts[i])?;
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
        let src_box = TrunAtom::default();
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
            data_offset: None,
            sample_count: 9,
            sample_sizes: Some(vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730]),
            sample_flags: Some(vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730]),
            first_sample_flags: None,
            sample_durations: Some(vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730]),
            sample_cts: Some(vec![1165, 11, 11, 8545, 10126, 10866, 9643, 9351, 7730]),
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
