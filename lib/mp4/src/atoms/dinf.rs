use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_atom, skip_bytes_to, write_atom_header_ext, Atom,
    AtomHeader, ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct DinfAtom {
    dref: DrefAtom,
}

impl Atom for DinfAtom {
    const FOUR_CC: FourCC = FourCC::new(b"dinf");

    fn size(&self) -> u64 {
        HEADER_SIZE + self.dref.size()
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for DinfAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut dref = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                DrefAtom::FOUR_CC => {
                    dref = Some(DrefAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if dref.is_none() {
            return Err(Error::BoxNotFound(DrefAtom::FOUR_CC));
        }
        let dref = dref.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { dref })
    }
}

impl<W: Write> WriteAtom<&mut W> for DinfAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;
        self.dref.write_atom(writer)?;
        Ok(self.size())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DrefAtom {
    pub version: u8,
    pub flags: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<UrlAtom>,
}

impl Default for DrefAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 0,
            url: Some(UrlAtom::default()),
        }
    }
}

impl Atom for DrefAtom {
    const FOUR_CC: FourCC = FourCC::new(b"dref");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE + 4;
        if let Some(ref url) = self.url {
            size += url.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("");
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for DrefAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut current = reader.seek(SeekFrom::Current(0))?;

        let (version, flags) = read_atom_header_ext(reader)?;
        let end = start + size;

        let mut url = None;

        let entry_count = reader.read_u32::<BigEndian>()?;
        for _i in 0..entry_count {
            if current >= end {
                break;
            }

            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                UrlAtom::FOUR_CC => {
                    url = Some(UrlAtom::read_atom(reader, s)?);
                }
                _ => {
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            url,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for DrefAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(1)?;

        if let Some(ref url) = self.url {
            url.write_atom(writer)?;
        }

        Ok(self.size())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UrlAtom {
    pub version: u8,
    pub flags: u32,
    pub location: String,
}

impl Default for UrlAtom {
    fn default() -> Self {
        Self {
            version: 0,
            flags: 1,
            location: String::default(),
        }
    }
}

impl Atom for UrlAtom {
    const FOUR_CC: FourCC = FourCC::new(b"url ");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + HEADER_EXT_SIZE;

        if !self.location.is_empty() {
            size += self.location.bytes().len() as u64 + 1;
        }

        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("location={}", self.location);
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for UrlAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        let location = if size - HEADER_SIZE - HEADER_EXT_SIZE > 0 {
            let buf_size = size - HEADER_SIZE - HEADER_EXT_SIZE - 1;
            let mut buf = vec![0u8; buf_size as usize];
            reader.read_exact(&mut buf)?;
            match String::from_utf8(buf) {
                Ok(t) => {
                    if t.len() != buf_size as usize {
                        return Err(Error::InvalidData("string too small"));
                    }
                    t
                }
                _ => String::default(),
            }
        } else {
            String::default()
        };

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            location,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for UrlAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        if !self.location.is_empty() {
            writer.write_all(self.location.as_bytes())?;
            writer.write_u8(0)?;
        }

        Ok(self.size())
    }
}
