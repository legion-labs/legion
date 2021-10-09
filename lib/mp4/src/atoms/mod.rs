//! All ISO-MP4 boxes (atoms) and operations.
//!
//! * [ISO/IEC 14496-12](https://en.wikipedia.org/wiki/MPEG-4_Part_12) - ISO Base Media File Format (`QuickTime`, MPEG-4, etc)
//! * [ISO/IEC 14496-14](https://en.wikipedia.org/wiki/MPEG-4_Part_14) - MP4 file format
//! * ISO/IEC 14496-17 - Streaming text format
//! * [ISO 23009-1](https://www.iso.org/standard/79329.html) -Dynamic adaptive streaming over HTTP (DASH)
//!
//! <http://mp4ra.org/#/atoms>
//!
//! Supported Atoms:
//! ftyp
//! moov
//!     mvhd
//!     trak
//!         tkhd
//!         mdia
//!             mdhd
//!             hdlr
//!             minf
//!                 stbl
//!                     stsd
//!                         avc1
//!                         hev1
//!                         mp4a
//!                         tx3g
//!                     stts
//!                     stsc
//!                     stsz
//!                     stss
//!                     stco
//!                     co64
//!                     ctts
//!                 dinf
//!                     dref
//!                 smhd
//!                 vmhd
//!         edts
//!             elst
//!     mvex
//!         mehd
//!         trex
//! emsg
//! moof
//!     mfhd
//!     traf
//!         tfhd
//!         trun
//! mdat
//! free
//!

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::convert::TryInto;
use std::io::{Read, Seek, SeekFrom, Write};

pub use bytes::Bytes;
pub use num_rational::Ratio;

use crate::{FourCC, Result};

pub(crate) mod avc1;
pub(crate) mod co64;
pub(crate) mod ctts;
pub(crate) mod dinf;
pub(crate) mod edts;
pub(crate) mod elst;
pub(crate) mod emsg;
pub(crate) mod ftyp;
pub(crate) mod hdlr;
pub(crate) mod hev1;
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
pub(crate) mod tfhd;
pub(crate) mod tkhd;
pub(crate) mod traf;
pub(crate) mod trak;
pub(crate) mod trex;
pub(crate) mod trun;
pub(crate) mod tx3g;
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
                size: size as u64,
            })
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<u64> {
        if self.size > u32::MAX as u64 {
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
    seeker.seek(SeekFrom::Current(size as i64))?;
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointU8(Ratio<u16>);

impl FixedPointU8 {
    pub fn new(val: u8) -> Self {
        Self(Ratio::new_raw(val as u16 * 0x100, 0x100))
    }

    pub fn new_raw(val: u16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(&self) -> u8 {
        self.0.to_integer() as u8
    }

    pub fn raw_value(&self) -> u16 {
        *self.0.numer()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointI8(Ratio<i16>);

impl FixedPointI8 {
    //pub fn new(val: i8) -> Self {
    //    Self(Ratio::new_raw(val as i16 * 0x100, 0x100))
    //}

    pub fn new_raw(val: i16) -> Self {
        Self(Ratio::new_raw(val, 0x100))
    }

    pub fn value(&self) -> i8 {
        self.0.to_integer() as i8
    }

    pub fn raw_value(&self) -> i16 {
        *self.0.numer()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FixedPointU16(Ratio<u32>);

impl FixedPointU16 {
    pub fn new(val: u16) -> Self {
        Self(Ratio::new_raw(val as u32 * 0x10000, 0x10000))
    }

    pub fn new_raw(val: u32) -> Self {
        Self(Ratio::new_raw(val, 0x10000))
    }

    pub fn value(&self) -> u16 {
        self.0.to_integer() as u16
    }

    pub fn raw_value(&self) -> u32 {
        *self.0.numer()
    }
}

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
