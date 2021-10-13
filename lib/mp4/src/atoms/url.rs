use byteorder::WriteBytesExt;
use serde::Serialize;
use std::io::{Read, Seek, Write};

use crate::{Error, FourCC, Result};

use super::{
    box_start, read_atom_header_ext, skip_bytes_to, write_atom_header_ext, Atom, AtomHeader,
    ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};

/// Url Atom
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
