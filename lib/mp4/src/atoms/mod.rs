/// All ISO-MP4 boxes (atoms) operations.
///
/// * [ISO/IEC 14496-12](https://en.wikipedia.org/wiki/MPEG-4_Part_12) - ISO Base Media File Format (`QuickTime`, MPEG-4, etc)
/// * [ISO/IEC 14496-12](https://mpeg.chiariglione.org/standards/mpeg-4/iso-base-media-file-format/text-isoiec-14496-12-5th-edition)
/// * [ISO/IEC 14496-14](https://en.wikipedia.org/wiki/MPEG-4_Part_14) - MP4 file format
/// * ISO/IEC 14496-17 - Streaming text format
/// * [ISO 23009-1](https://www.iso.org/standard/79329.html) -Dynamic adaptive streaming over HTTP (DASH)
/// * [Quicktime Documentation](https://developer.apple.com/library/archive/documentation/QuickTime/QTFF/QTFFChap2/qtff2.html)
///
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

pub use bytes::Bytes;
pub use num_rational::Ratio;

use crate::{FourCC, Result};

pub(crate) mod avc1;
pub(crate) mod co64;
pub(crate) mod ctts;
pub(crate) mod dinf;
pub(crate) mod dref;
pub(crate) mod edts;
pub(crate) mod elst;
pub(crate) mod emsg;
pub(crate) mod ftyp;
pub(crate) mod hdlr;
pub(crate) mod hev1;
pub(crate) mod mdat;
pub(crate) mod mdhd;
pub(crate) mod mdia;
pub(crate) mod mehd;
pub(crate) mod mfhd;
pub(crate) mod minf;
pub(crate) mod moof;
pub(crate) mod moov;
pub(crate) mod mp4a;
pub(crate) mod mvex;
pub(crate) mod mvhd;
pub(crate) mod smhd;
pub(crate) mod stbl;
pub(crate) mod stco;
pub(crate) mod stsc;
pub(crate) mod stsd;
pub(crate) mod stss;
pub(crate) mod stsz;
pub(crate) mod stts;
pub(crate) mod tfdt;
pub(crate) mod tfhd;
pub(crate) mod tkhd;
pub(crate) mod traf;
pub(crate) mod trak;
pub(crate) mod trex;
pub(crate) mod trun;
pub(crate) mod tx3g;
pub(crate) mod url;
pub(crate) mod vmhd;
pub(crate) mod vp09;
pub(crate) mod vpcc;

pub const HEADER_SIZE: u64 = 8;
// const HEADER_LARGE_SIZE: u64 = 16;
pub const HEADER_EXT_SIZE: u64 = 4;

pub trait Atom: Sized {
    const FOUR_CC: FourCC;

    fn size(&self) -> u64;
    fn to_json(&self) -> Result<String>;
    fn summary(&self) -> Result<String>;
}

pub trait ReadAtom<T>: Sized {
    fn read_atom(_: T, size: u64) -> Result<Self>;
}

pub trait WriteAtom<T>: Sized {
    fn write_atom(&self, _: T) -> Result<u64>;
}

#[derive(Debug, Clone, Copy)]
pub struct AtomHeader {
    pub name: FourCC,
    pub size: u64,
}

impl AtomHeader {
    pub fn new<A: Atom>(mp4_box: &A) -> Self {
        Self {
            name: A::FOUR_CC,
            size: mp4_box.size(),
        }
    }

    // TODO: if size is 0, then this box is the last one in the file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Create and read to buf.
        let mut buf = [0u8; 8]; // 8 bytes for box header.
        reader.read_exact(&mut buf)?;

        // Get size.
        let s = buf[0..4].try_into().unwrap();
        let size = u32::from_be_bytes(s);

        // Get box type string.
        let t = buf[4..8].try_into().unwrap();
        let typ = u32::from_be_bytes(t).into();

        // Get largesize if size is 1
        if size == 1 {
            reader.read_exact(&mut buf)?;
            let largesize = u64::from_be_bytes(buf);

            Ok(Self {
                name: typ,
                size: largesize - HEADER_SIZE,
            })
        } else {
            Ok(Self {
                name: typ,
                size: u64::from(size),
            })
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<u64> {
        if self.size > u64::from(u32::MAX) {
            writer.write_u32::<BigEndian>(1)?;
            writer.write_u32::<BigEndian>(self.name.into())?;
            writer.write_u64::<BigEndian>(self.size)?;
            Ok(16)
        } else {
            writer.write_u32::<BigEndian>(self.size as u32)?;
            writer.write_u32::<BigEndian>(self.name.into())?;
            Ok(8)
        }
    }
}

pub fn read_atom_header_ext<R: Read>(reader: &mut R) -> Result<(u8, u32)> {
    let version = reader.read_u8()?;
    let flags = reader.read_u24::<BigEndian>()?;
    Ok((version, flags))
}

pub fn write_atom_header_ext<W: Write>(w: &mut W, v: u8, f: u32) -> Result<u64> {
    w.write_u8(v)?;
    w.write_u24::<BigEndian>(f)?;
    Ok(4)
}

pub fn box_start<R: Seek>(seeker: &mut R) -> Result<u64> {
    Ok(seeker.seek(SeekFrom::Current(0))? - HEADER_SIZE)
}

pub fn skip_bytes<S: Seek>(seeker: &mut S, size: u64) -> Result<()> {
    let size = size
        .try_into()
        .expect("skip size expected to be lower than i64::MAX");
    seeker.seek(SeekFrom::Current(size))?;
    Ok(())
}

pub fn skip_bytes_to<S: Seek>(seeker: &mut S, pos: u64) -> Result<()> {
    seeker.seek(SeekFrom::Start(pos))?;
    Ok(())
}

pub fn skip_atom<S: Seek>(seeker: &mut S, size: u64) -> Result<()> {
    let start = box_start(seeker)?;
    skip_bytes_to(seeker, start + size)?;
    Ok(())
}

pub fn write_zeros<W: Write>(writer: &mut W, size: u64) -> Result<()> {
    for _ in 0..size {
        writer.write_u8(0)?;
    }
    Ok(())
}

/// U8.U8 fixed point representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointU8(Ratio<u16>);

impl FixedPointU8 {
    pub fn new(val: u8) -> Self {
        Self(Ratio::new_raw(u16::from(val) * 0x100, 0x100))
    }

    pub fn new_raw(val: u16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(self) -> u8 {
        self.0.to_integer() as u8
    }

    pub fn raw_value(self) -> u16 {
        *self.0.numer()
    }
}

/// I8.U8 fixed point representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointI8(Ratio<i16>);

impl FixedPointI8 {
    //pub fn new(val: i8) -> Self {
    //    Self(Ratio::new_raw(val as i16 * 0x100, 0x100))
    //}

    pub fn new_raw(val: i16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(self) -> i8 {
        self.0.to_integer() as i8
    }

    pub fn raw_value(self) -> i16 {
        *self.0.numer()
    }
}

/// U16.U16 fixed point representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointU16(Ratio<u32>);

impl FixedPointU16 {
    pub fn new(val: u16) -> Self {
        Self(Ratio::new_raw(u32::from(val) * 0x10000, 0x10000))
    }

    pub fn new_raw(val: u32) -> Self {
        Self(Ratio::new_raw(val, 0x10000))
    }

    pub fn value(self) -> u16 {
        self.0.to_integer() as u16
    }

    pub fn raw_value(self) -> u32 {
        *self.0.numer()
    }
}

/// provides a transformation matrix for the video; (u,v,w) are restricted here to (0,0,1), hex values (0,0,0x40000000)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Matrix {
    pub a: i32,
    pub b: i32,
    pub u: i32,
    pub c: i32,
    pub d: i32,
    pub v: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            a: 0x00010000,
            b: 0,
            u: 0,
            c: 0,
            d: 0x00010000,
            v: 0,
            x: 0,
            y: 0,
            w: 0x40000000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SampleIsLeading {
    /// the leading nature of this sample is unknown
    Unknown,
    /// this sample is a leading sample that has a dependency
    /// before the referenced I-picture (and is therefore not decodable)
    LeadingWithDep,
    /// this sample is not a leading sample
    NotLeading,
    /// this sample is a leading sample that has no dependency
    /// before the referenced I-picture (and is therefore decodable)
    LeadingWithoutDep,
}

impl From<SampleIsLeading> for u32 {
    fn from(value: SampleIsLeading) -> Self {
        match value {
            SampleIsLeading::Unknown => 0,
            SampleIsLeading::LeadingWithDep => 1,
            SampleIsLeading::NotLeading => 2,
            SampleIsLeading::LeadingWithoutDep => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SampleDependsOn {
    /// the dependency of this sample is unknown
    Unknown,
    /// this sample does depend on others (not an I picture)
    Others,
    /// this sample does not depend on others (I picture)
    None,
}

impl From<SampleDependsOn> for u32 {
    fn from(value: SampleDependsOn) -> Self {
        match value {
            SampleDependsOn::Unknown => 0,
            SampleDependsOn::Others => 1,
            SampleDependsOn::None => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SampleIsDependedOn {
    /// the dependency of other samples on this sample is unknown
    Unknown,
    /// other samples may depend on this one (not disposable)
    Others,
    /// no other sample depends on this one (disposable)
    None,
}

impl From<SampleIsDependedOn> for u32 {
    fn from(value: SampleIsDependedOn) -> Self {
        match value {
            SampleIsDependedOn::Unknown => 0,
            SampleIsDependedOn::Others => 1,
            SampleIsDependedOn::None => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SampleHasRedundancy {
    /// it is unknown whether there is redundant coding in this sample
    Unknown,
    /// there is redundant coding in this sample
    Redundant,
    /// there is no redundant coding in this sample
    NotRedundant,
}

impl From<SampleHasRedundancy> for u32 {
    fn from(value: SampleHasRedundancy) -> Self {
        match value {
            SampleHasRedundancy::Unknown => 0,
            SampleHasRedundancy::Redundant => 1,
            SampleHasRedundancy::NotRedundant => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SampleFlags {
    pub is_leading: SampleIsLeading,
    pub sample_depends_on: SampleDependsOn,
    pub sample_is_depdended_on: SampleIsDependedOn,
    pub sample_has_redundancy: SampleHasRedundancy,
    pub sample_padding_value: u8,
    pub sample_is_non_sync_sample: bool,
    pub sample_degradation_priority: u16,
}

impl Default for SampleFlags {
    fn default() -> Self {
        Self {
            is_leading: SampleIsLeading::Unknown,
            sample_depends_on: SampleDependsOn::Unknown,
            sample_is_depdended_on: SampleIsDependedOn::Unknown,
            sample_has_redundancy: SampleHasRedundancy::Unknown,
            sample_padding_value: 0,
            sample_is_non_sync_sample: false,
            sample_degradation_priority: 0,
        }
    }
}

impl SampleFlags {
    // 4 bits reserved
    // 2 bits is_leading
    const IS_LEADING_OFFSET: u32 = 26;
    const IS_LEADING_MASK: u32 = 0x3;
    // 2 bits sample_depends_on
    const SAMPLE_DEPENDS_ON_OFFSET: u32 = 24;
    const SAMPLE_DEPENDS_ON_MASK: u32 = 0x3;
    // 2 bits sample_depends_on
    const SAMPLE_IS_DEPENDED_ON_OFFSET: u32 = 22;
    const SAMPLE_IS_DEPENDED_ON_MASK: u32 = 0x3;
    // 2 bits sample_has_redundancy
    const SAMPLE_HAS_REDUNDANCY_OFFSET: u32 = 20;
    const SAMPLE_HAS_REDUNDANCY_MASK: u32 = 0x3;
    // 3 bits sample_padding_value
    const SAMPLE_PADDING_VALUE_OFFSET: u32 = 17;
    const SAMPLE_PADDING_VALUE_MASK: u32 = 0x7;
    // 1 bits sample_is_non_sync_sample
    const SAMPLE_IS_NON_SYNC_SAMPLE_OFFSET: u32 = 16;
    const SAMPLE_IS_NON_SYNC_SAMPLE_MASK: u32 = 0x1;
    // 16 bits sample_degradation_priority
    const SAMPLE_DEGRADATION_PRIORITY_MASK: u32 = 0xFF_FF;
}

impl From<SampleFlags> for u32 {
    fn from(value: SampleFlags) -> Self {
        // 4 bit reserved
        (Self::from(value.is_leading) << SampleFlags::IS_LEADING_OFFSET) // 2bits
            | (Self::from(value.sample_depends_on) << SampleFlags::SAMPLE_DEPENDS_ON_OFFSET) // 2bits
            | (Self::from(value.sample_is_depdended_on) << SampleFlags::SAMPLE_IS_DEPENDED_ON_OFFSET) // 2bits
            | (Self::from(value.sample_has_redundancy) << SampleFlags::SAMPLE_HAS_REDUNDANCY_OFFSET) // 2bits
            | (Self::from(value.sample_padding_value) << SampleFlags::SAMPLE_PADDING_VALUE_OFFSET) // 3bits
            | ((value.sample_is_non_sync_sample as Self) << SampleFlags::SAMPLE_IS_NON_SYNC_SAMPLE_OFFSET) // 1bits
            | Self::from(value.sample_degradation_priority) // remaining 16 bits
    }
}

impl From<u32> for SampleFlags {
    fn from(value: u32) -> Self {
        let is_leading = match (value >> Self::IS_LEADING_OFFSET) & Self::IS_LEADING_MASK {
            1 => SampleIsLeading::LeadingWithDep,
            2 => SampleIsLeading::NotLeading,
            3 => SampleIsLeading::LeadingWithoutDep,
            _ => SampleIsLeading::Unknown,
        };

        let sample_depends_on =
            match (value >> Self::SAMPLE_DEPENDS_ON_OFFSET) & Self::SAMPLE_DEPENDS_ON_MASK {
                1 => SampleDependsOn::Others,
                2 => SampleDependsOn::None,
                _ => SampleDependsOn::Unknown,
            };

        let sample_is_depdended_on = match (value >> Self::SAMPLE_IS_DEPENDED_ON_OFFSET)
            & Self::SAMPLE_IS_DEPENDED_ON_MASK
        {
            1 => SampleIsDependedOn::Others,
            2 => SampleIsDependedOn::None,
            _ => SampleIsDependedOn::Unknown,
        };

        let sample_has_redundancy = match (value >> Self::SAMPLE_HAS_REDUNDANCY_OFFSET)
            & Self::SAMPLE_HAS_REDUNDANCY_MASK
        {
            1 => SampleHasRedundancy::Redundant,
            2 => SampleHasRedundancy::NotRedundant,
            _ => SampleHasRedundancy::Unknown,
        };

        Self {
            is_leading,
            sample_depends_on,
            sample_is_depdended_on,
            sample_has_redundancy,
            sample_padding_value: ((value >> Self::SAMPLE_PADDING_VALUE_OFFSET)
                & Self::SAMPLE_PADDING_VALUE_MASK) as u8,
            sample_is_non_sync_sample: ((value >> Self::SAMPLE_IS_NON_SYNC_SAMPLE_OFFSET)
                & Self::SAMPLE_IS_NON_SYNC_SAMPLE_MASK)
                != 0,
            sample_degradation_priority: (value & Self::SAMPLE_DEGRADATION_PRIORITY_MASK) as u16,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // we need to conform to the serializer interface
mod value_u32 {
    use super::FixedPointU16;
    use serde::{self, Serializer};

    pub fn serialize<S>(fixed: &FixedPointU16, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(fixed.value())
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // we need to conform to the serializer interface
mod value_i16 {
    use super::FixedPointI8;
    use serde::{self, Serializer};

    pub fn serialize<S>(fixed: &FixedPointI8, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i8(fixed.value())
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)] // we need to conform to the serializer interface
mod value_u8 {
    use super::FixedPointU8;
    use serde::{self, Serializer};

    pub fn serialize<S>(fixed: &FixedPointU8, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(fixed.value())
    }
}

#[cfg(test)]
mod tests {
    use crate::atoms::SampleDependsOn;

    use super::SampleFlags;

    #[test]
    fn sample_flags() {
        let src = SampleFlags {
            sample_is_non_sync_sample: true,
            sample_depends_on: SampleDependsOn::Others,
            ..SampleFlags::default()
        };
        assert_eq!(u32::from(src), 0x1010000);
        assert_eq!(SampleFlags::from(0x1010000), src);
        let src = SampleFlags {
            sample_depends_on: SampleDependsOn::None,
            ..SampleFlags::default()
        };
        assert_eq!(u32::from(src), 0x2000000);
        assert_eq!(SampleFlags::from(0x2000000), src);
    }
}
