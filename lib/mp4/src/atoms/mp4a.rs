use std::io::{Read, Seek, SeekFrom, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes, skip_bytes_to, value_u32, write_atom_header_ext,
    Atom, AtomHeader, FixedPointU16, ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{AacConfig, Error, FourCC, Result};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Mp4aAtom {
    pub data_reference_index: u16,
    pub channelcount: u16,
    pub samplesize: u16,

    #[serde(with = "value_u32")]
    pub samplerate: FixedPointU16,
    pub esds: Option<EsdsAtom>,
}

impl Default for Mp4aAtom {
    fn default() -> Self {
        Self {
            data_reference_index: 0,
            channelcount: 2,
            samplesize: 16,
            samplerate: FixedPointU16::new(48000),
            esds: Some(EsdsAtom::default()),
        }
    }
}

impl Mp4aAtom {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            data_reference_index: 1,
            channelcount: config.chan_conf as u16,
            samplesize: 16,
            samplerate: FixedPointU16::new(config.freq_index.freq() as u16),
            esds: Some(EsdsAtom::new(config)),
        }
    }
}

impl Atom for Mp4aAtom {
    const FOUR_CC: FourCC = FourCC::new(b"mp4a");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + 8 + 20;
        if let Some(ref esds) = self.esds {
            size += esds.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "channel_count={} sample_size={} sample_rate={}",
            self.channelcount,
            self.samplesize,
            self.samplerate.value()
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for Mp4aAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        reader.read_u32::<BigEndian>()?; // reserved
        reader.read_u16::<BigEndian>()?; // reserved
        let data_reference_index = reader.read_u16::<BigEndian>()?;

        reader.read_u64::<BigEndian>()?; // reserved
        let channelcount = reader.read_u16::<BigEndian>()?;
        let samplesize = reader.read_u16::<BigEndian>()?;
        reader.read_u32::<BigEndian>()?; // pre-defined, reserved
        let samplerate = FixedPointU16::new_raw(reader.read_u32::<BigEndian>()?);

        let current = reader.seek(SeekFrom::Current(0))?;
        let mut esds = None;
        if current < start + size {
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            if name == EsdsAtom::FOUR_CC {
                esds = Some(EsdsAtom::read_atom(reader, s)?);
            }
        }
        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            data_reference_index,
            channelcount,
            samplesize,
            samplerate,
            esds,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for Mp4aAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.data_reference_index)?;

        writer.write_u64::<BigEndian>(0)?; // reserved
        writer.write_u16::<BigEndian>(self.channelcount)?;
        writer.write_u16::<BigEndian>(self.samplesize)?;
        writer.write_u32::<BigEndian>(0)?; // reserved
        writer.write_u32::<BigEndian>(self.samplerate.raw_value())?;

        if let Some(ref esds) = self.esds {
            esds.write_atom(writer)?;
        }

        Ok(self.size())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct EsdsAtom {
    pub version: u8,
    pub flags: u32,
    pub es_desc: ESDescriptor,
}

impl EsdsAtom {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            version: 0,
            flags: 0,
            es_desc: ESDescriptor::new(config),
        }
    }
}

impl Atom for EsdsAtom {
    const FOUR_CC: FourCC = FourCC::new(b"esds");

    fn size(&self) -> u64 {
        HEADER_SIZE
            + HEADER_EXT_SIZE
            + 1
            + u64::from(size_of_length(ESDescriptor::desc_size()))
            + u64::from(ESDescriptor::desc_size())
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for EsdsAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let mut es_desc = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x03 => {
                    es_desc = Some(ESDescriptor::read_desc(reader, desc_size)?);
                }
                _ => break,
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        if es_desc.is_none() {
            return Err(Error::InvalidData("ESDescriptor not found"));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            es_desc: es_desc.unwrap(),
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for EsdsAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        self.es_desc.write_desc(writer)?;

        Ok(self.size())
    }
}

trait Descriptor: Sized {
    fn desc_tag() -> u8;
    fn desc_size() -> u32;
}

trait ReadDesc<T>: Sized {
    fn read_desc(_: T, size: u32) -> Result<Self>;
}

trait WriteDesc<T>: Sized {
    fn write_desc(&self, _: T) -> Result<u32>;
}

fn read_desc<R: Read>(reader: &mut R) -> Result<(u8, u32)> {
    let tag = reader.read_u8()?;

    let mut size: u32 = 0;
    for _ in 0..4 {
        let b = reader.read_u8()?;
        size = (size << 7) | u32::from(b & 0x7F);
        if b & 0x80 == 0 {
            break;
        }
    }

    Ok((tag, size))
}

fn size_of_length(size: u32) -> u32 {
    match size {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        _ => 4,
    }
}

fn write_desc<W: Write>(writer: &mut W, tag: u8, size: u32) -> Result<u64> {
    writer.write_u8(tag)?;

    if u64::from(size) > u64::from(std::u32::MAX) {
        return Err(Error::InvalidData("invalid descriptor length range"));
    }

    let nbytes = size_of_length(size);

    for i in 0..nbytes {
        let mut b = (size >> ((nbytes - i - 1) * 7)) as u8 & 0x7F;
        if i < nbytes - 1 {
            b |= 0x80;
        }
        writer.write_u8(b)?;
    }

    Ok(u64::from(nbytes) + 1)
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct ESDescriptor {
    pub es_id: u16,

    pub dec_config: DecoderConfigDescriptor,
    pub sl_config: SLConfigDescriptor,
}

impl ESDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            es_id: 1,
            dec_config: DecoderConfigDescriptor::new(config),
            sl_config: SLConfigDescriptor::new(),
        }
    }
}

impl Descriptor for ESDescriptor {
    fn desc_tag() -> u8 {
        0x03
    }

    fn desc_size() -> u32 {
        3 + 1
            + size_of_length(DecoderConfigDescriptor::desc_size())
            + DecoderConfigDescriptor::desc_size()
            + 1
            + size_of_length(SLConfigDescriptor::desc_size())
            + SLConfigDescriptor::desc_size()
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for ESDescriptor {
    fn read_desc(reader: &mut R, size: u32) -> Result<Self> {
        let start = reader.seek(SeekFrom::Current(0))?;

        let es_id = reader.read_u16::<BigEndian>()?;
        reader.read_u8()?; // XXX flags must be 0

        let mut dec_config = None;
        let mut sl_config = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + u64::from(size);
        while current < end {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x04 => {
                    dec_config = Some(DecoderConfigDescriptor::read_desc(reader, desc_size)?);
                }
                0x06 => {
                    sl_config = Some(SLConfigDescriptor::read_desc(reader, desc_size)?);
                }
                _ => {
                    skip_bytes(reader, u64::from(desc_size))?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        Ok(Self {
            es_id,
            dec_config: dec_config.unwrap_or_default(),
            sl_config: sl_config.unwrap_or_default(),
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for ESDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        writer.write_u16::<BigEndian>(self.es_id)?;
        writer.write_u8(0)?;

        self.dec_config.write_desc(writer)?;
        self.sl_config.write_desc(writer)?;

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DecoderConfigDescriptor {
    pub object_type_indication: u8,
    pub stream_type: u8,
    pub up_stream: u8,
    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,

    pub dec_specific: DecoderSpecificDescriptor,
}

impl DecoderConfigDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            object_type_indication: 0x40, // XXX AAC
            stream_type: 0x05,            // XXX Audio
            up_stream: 0,
            buffer_size_db: 0,
            max_bitrate: config.bitrate, // XXX
            avg_bitrate: config.bitrate,
            dec_specific: DecoderSpecificDescriptor::new(config),
        }
    }
}

impl Descriptor for DecoderConfigDescriptor {
    fn desc_tag() -> u8 {
        0x04
    }

    fn desc_size() -> u32 {
        13 + 1
            + size_of_length(DecoderSpecificDescriptor::desc_size())
            + DecoderSpecificDescriptor::desc_size()
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderConfigDescriptor {
    fn read_desc(reader: &mut R, size: u32) -> Result<Self> {
        let start = reader.seek(SeekFrom::Current(0))?;

        let object_type_indication = reader.read_u8()?;
        let byte_a = reader.read_u8()?;
        let stream_type = (byte_a & 0xFC) >> 2;
        let up_stream = byte_a & 0x02;
        let buffer_size_db = reader.read_u24::<BigEndian>()?;
        let max_bitrate = reader.read_u32::<BigEndian>()?;
        let avg_bitrate = reader.read_u32::<BigEndian>()?;

        let mut dec_specific = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + u64::from(size);
        while current < end {
            let (desc_tag, desc_size) = read_desc(reader)?;
            match desc_tag {
                0x05 => {
                    dec_specific = Some(DecoderSpecificDescriptor::read_desc(reader, desc_size)?);
                }
                _ => {
                    skip_bytes(reader, u64::from(desc_size))?;
                }
            }
            current = reader.seek(SeekFrom::Current(0))?;
        }

        Ok(Self {
            object_type_indication,
            stream_type,
            up_stream,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            dec_specific: dec_specific.unwrap_or_default(),
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        writer.write_u8(self.object_type_indication)?;
        writer.write_u8((self.stream_type << 2) + (self.up_stream & 0x02) + 1)?; // 1 reserved
        writer.write_u24::<BigEndian>(self.buffer_size_db)?;
        writer.write_u32::<BigEndian>(self.max_bitrate)?;
        writer.write_u32::<BigEndian>(self.avg_bitrate)?;

        self.dec_specific.write_desc(writer)?;

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DecoderSpecificDescriptor {
    pub profile: u8,
    pub freq_index: u8,
    pub chan_conf: u8,
}

impl DecoderSpecificDescriptor {
    pub fn new(config: &AacConfig) -> Self {
        Self {
            profile: config.profile as u8,
            freq_index: config.freq_index as u8,
            chan_conf: config.chan_conf as u8,
        }
    }
}

impl Descriptor for DecoderSpecificDescriptor {
    fn desc_tag() -> u8 {
        0x05
    }

    fn desc_size() -> u32 {
        2
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for DecoderSpecificDescriptor {
    fn read_desc(reader: &mut R, _size: u32) -> Result<Self> {
        let byte_a = reader.read_u8()?;
        let byte_b = reader.read_u8()?;
        let profile = byte_a >> 3;
        let freq_index = ((byte_a & 0x07) << 1) + (byte_b >> 7);
        let chan_conf = (byte_b >> 3) & 0x0F;

        Ok(Self {
            profile,
            freq_index,
            chan_conf,
        })
    }
}

impl<W: Write> WriteDesc<&mut W> for DecoderSpecificDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size)?;

        writer.write_u8((self.profile << 3) + (self.freq_index >> 1))?;
        writer.write_u8((self.freq_index << 7) + (self.chan_conf << 3))?;

        Ok(size)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct SLConfigDescriptor {}

impl SLConfigDescriptor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Descriptor for SLConfigDescriptor {
    fn desc_tag() -> u8 {
        0x06
    }

    fn desc_size() -> u32 {
        1
    }
}

impl<R: Read + Seek> ReadDesc<&mut R> for SLConfigDescriptor {
    fn read_desc(reader: &mut R, _size: u32) -> Result<Self> {
        reader.read_u8()?; // pre-defined

        Ok(Self {})
    }
}

impl<W: Write> WriteDesc<&mut W> for SLConfigDescriptor {
    fn write_desc(&self, writer: &mut W) -> Result<u32> {
        let size = Self::desc_size();
        write_desc(writer, Self::desc_tag(), size - 1)?;

        writer.write_u8(0)?; // pre-defined
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_mp4a() {
        let src_box = Mp4aAtom {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: FixedPointU16::new(48000),
            esds: Some(EsdsAtom {
                version: 0,
                flags: 0,
                es_desc: ESDescriptor {
                    es_id: 2,
                    dec_config: DecoderConfigDescriptor {
                        object_type_indication: 0x40,
                        stream_type: 0x05,
                        up_stream: 0,
                        buffer_size_db: 0,
                        max_bitrate: 67695,
                        avg_bitrate: 67695,
                        dec_specific: DecoderSpecificDescriptor {
                            profile: 2,
                            freq_index: 3,
                            chan_conf: 1,
                        },
                    },
                    sl_config: SLConfigDescriptor::default(),
                },
            }),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, Mp4aAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = Mp4aAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_mp4a_no_esds() {
        let src_box = Mp4aAtom {
            data_reference_index: 1,
            channelcount: 2,
            samplesize: 16,
            samplerate: FixedPointU16::new(48000),
            esds: None,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, Mp4aAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = Mp4aAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
