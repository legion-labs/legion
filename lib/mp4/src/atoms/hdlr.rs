use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{
    box_start, read_atom_header_ext, skip_bytes, skip_bytes_to, write_atom_header_ext, Atom,
    AtomHeader, ReadAtom, WriteAtom, HEADER_EXT_SIZE, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct HdlrAtom {
    pub version: u8,
    pub flags: u32,
    pub handler_type: FourCC,
    pub name: String,
}

// Handler Reference Atom
impl Atom for HdlrAtom {
    const FOUR_CC: FourCC = FourCC::new(b"hdlr");

    fn size(&self) -> u64 {
        HEADER_SIZE + HEADER_EXT_SIZE + 20 + self.name.len() as u64 + 1
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!(
            "handler_type={} name={}",
            self.handler_type.to_string(),
            self.name
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for HdlrAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let (version, flags) = read_atom_header_ext(reader)?;

        reader.read_u32::<BigEndian>()?; // pre-defined
        let handler = reader.read_u32::<BigEndian>()?;

        skip_bytes(reader, 12)?; // reserved

        let buf_size = size - HEADER_SIZE - HEADER_EXT_SIZE - 20 - 1;
        let mut buf = vec![0u8; buf_size as usize];
        reader.read_exact(&mut buf)?;

        let handler_string = match String::from_utf8(buf) {
            Ok(t) => {
                if t.len() != buf_size as usize {
                    return Err(Error::InvalidData("string too small"));
                }
                t
            }
            _ => String::from("null"),
        };

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            version,
            flags,
            handler_type: From::from(handler),
            name: handler_string,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for HdlrAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        write_atom_header_ext(writer, self.version, self.flags)?;

        writer.write_u32::<BigEndian>(0)?; // pre-defined
        writer.write_u32::<BigEndian>((&self.handler_type).into())?;

        // 12 bytes reserved
        for _ in 0..3 {
            writer.write_u32::<BigEndian>(0)?;
        }

        writer.write_all(self.name.as_bytes())?;
        writer.write_u8(0)?;

        Ok(self.size())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::atoms::AtomHeader;

    #[test]
    fn test_hdlr() {
        let src_box = HdlrAtom {
            version: 0,
            flags: 0,
            handler_type: str::parse::<FourCC>("vide").unwrap(),
            name: String::from("VideoHandler"),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, HdlrAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = HdlrAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
