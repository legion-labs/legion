use std::io::{Read, Seek, Write};

use bitflags::bitflags;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, SampleFlags, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

bitflags! {
    #[derive(Default, Serialize)]
    struct TrackFragmentFlags: u32 {
        /// indicates the presence of the base-data-offset field. This provides an explicit anchor for the data offsets
        /// in each track run (see below). If not provided and if the default-base-is-moof flag is not set,
        /// the base-data-offset for the first track in the movie fragment is the position of the first byte of
        /// the enclosing Movie Fragment Box, and for second and subsequent track fragments,
        /// the default is the end of the data defined by the preceding track fragment.
        /// Fragments 'inheriting' their offset in this way must all use the same data-reference
        /// (i.e., the data for these tracks must be in the same file)
        const BASE_DATA_OFFSET_PRESENT = 0x000001;
        /// indicates the presence of this field, which over-rides, in this fragment, the default set up in the Track Extends Box.
        const SAMPLE_DESCRIPTION_INDEX_PRESENT = 0x000002;
        /// indicates the presence of this field
        const DEFAULT_SAMPLE_DURATION_PRESENT = 0x000008;
        /// indicates the presence of this field
        const DEFAULT_SAMPLE_SIZE_PRESENT = 0x000010;
        /// indicates the presence of this field
        const DEFAULT_SAMPLE_FLAGS_PRESENT = 0x000020;
        /// this indicates that the duration provided in either default-sample-duration, or by the default-duration
        /// in the Track Extends Box, is empty, i.e. that there are no samples for this time interval.
        /// It is an error to make a presentation that has both edit lists in the Movie Box, and empty-duration fragments.
        const DURATION_IS_EMPTY = 0x010000;
        /// if base-data-offset-present is 1, this flag is ignored. If base-data-offset-present is zero,
        /// this indicates that the base-data-offset for this track fragment is the position of the first byte
        /// of the enclosing Movie Fragment Box. Support for the default-base-is-moof flag is required under the ‘iso5’ brand,
        /// and it shall not be used in brands or compatible brands earlier than iso5.
        const DEFAULT_BASE_IS_MOOF = 0x020000;
    }
}

/// Track Fragment Header Atom
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct TfhdAtom {
    pub version: u8,
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<SampleFlags>,
    pub duration_is_empty: bool,
    pub default_base_is_moof: bool,
}

impl From<&TfhdAtom> for TrackFragmentFlags {
    fn from(value: &TfhdAtom) -> Self {
        let mut flags = Self::empty();
        flags.set(Self::DURATION_IS_EMPTY, value.duration_is_empty);
        flags.set(Self::DEFAULT_BASE_IS_MOOF, value.default_base_is_moof);
        flags.set(
            Self::BASE_DATA_OFFSET_PRESENT,
            value.base_data_offset.is_some(),
        );
        flags.set(
            Self::SAMPLE_DESCRIPTION_INDEX_PRESENT,
            value.sample_description_index.is_some(),
        );
        flags.set(
            Self::DEFAULT_SAMPLE_DURATION_PRESENT,
            value.default_sample_duration.is_some(),
        );
        flags.set(
            Self::DEFAULT_SAMPLE_SIZE_PRESENT,
            value.default_sample_size.is_some(),
        );
        flags.set(
            Self::DEFAULT_SAMPLE_FLAGS_PRESENT,
            value.default_sample_flags.is_some(),
        );
        flags
    }
}

impl Atom for TfhdAtom {
    const FOUR_CC: FourCC = FourCC::new(b"tfhd");

    fn size(&self) -> u64 {
        HEADER_SIZE
            + HEADER_EXT_SIZE
            + 4 // track id
            + self.base_data_offset.map_or(0, |_| 8)
            + self.sample_description_index.map_or(0, |_| 4)
            + self.default_sample_duration.map_or(0, |_| 4)
            + self.default_sample_size.map_or(0, |_| 4)
            + self.default_sample_flags.map_or(0, |_| 4)
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("track_id={}", self.track_id);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TfhdAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;
        let track_id = reader.read_u32::<BigEndian>()?;
        let flags =
            TrackFragmentFlags::from_bits(flags).ok_or(Error::InvalidData("invalid tf_flags"))?;
        let (duration_is_empty, default_base_is_moof) = (
            flags.contains(TrackFragmentFlags::DURATION_IS_EMPTY),
            flags.contains(TrackFragmentFlags::DEFAULT_BASE_IS_MOOF),
        );
        let base_data_offset = if flags.contains(TrackFragmentFlags::BASE_DATA_OFFSET_PRESENT) {
            Some(reader.read_u64::<BigEndian>()?)
        } else {
            None
        };
        let sample_description_index =
            if flags.contains(TrackFragmentFlags::SAMPLE_DESCRIPTION_INDEX_PRESENT) {
                Some(reader.read_u32::<BigEndian>()?)
            } else {
                None
            };
        let default_sample_duration =
            if flags.contains(TrackFragmentFlags::DEFAULT_SAMPLE_DURATION_PRESENT) {
                Some(reader.read_u32::<BigEndian>()?)
            } else {
                None
            };
        let default_sample_size = if flags.contains(TrackFragmentFlags::DEFAULT_SAMPLE_SIZE_PRESENT)
        {
            Some(reader.read_u32::<BigEndian>()?)
        } else {
            None
        };
        let default_sample_flags =
            if flags.contains(TrackFragmentFlags::DEFAULT_SAMPLE_FLAGS_PRESENT) {
                Some(reader.read_u32::<BigEndian>()?.into())
            } else {
                None
            };

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            track_id,
            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
            duration_is_empty,
            default_base_is_moof,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for TfhdAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        let flags = TrackFragmentFlags::from(self);
        write_atom_header_ext(writer, self.version, flags.bits())?;
        writer.write_u32::<BigEndian>(self.track_id)?;
        if let Some(base_data_offset) = self.base_data_offset {
            writer.write_u64::<BigEndian>(base_data_offset)?;
        }
        if let Some(sample_description_index) = self.sample_description_index {
            writer.write_u32::<BigEndian>(sample_description_index)?;
        }
        if let Some(default_sample_duration) = self.default_sample_duration {
            writer.write_u32::<BigEndian>(default_sample_duration)?;
        }
        if let Some(default_sample_size) = self.default_sample_size {
            writer.write_u32::<BigEndian>(default_sample_size)?;
        }
        if let Some(default_sample_flags) = self.default_sample_flags {
            writer.write_u32::<BigEndian>(default_sample_flags.into())?;
        }

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_tfhd() {
        let src_box = TfhdAtom {
            track_id: 1,
            ..TfhdAtom::default()
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TfhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TfhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_tfhd_base_is_moof_with_default_sample_flags() {
        let src_box = TfhdAtom {
            track_id: 1,
            default_base_is_moof: true,
            default_sample_flags: Some(0x1010000.into()),
            ..TfhdAtom::default()
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TfhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TfhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_tfhd_base_is_moof_with_default_sample_duration() {
        let src_box = TfhdAtom {
            track_id: 1,
            default_base_is_moof: true,
            default_sample_duration: Some(0x1),
            ..TfhdAtom::default()
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TfhdAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TfhdAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
