use std::io::{Read, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::Serialize;

use super::{box_start, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE};
use crate::{Error, FourCC, Result};

/// File Type Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct FtypAtom {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
}

impl Atom for FtypAtom {
    const FOUR_CC: FourCC = FourCC::new(b"ftyp");

    fn size(&self) -> u64 {
        HEADER_SIZE + 8 + (4 * self.compatible_brands.len() as u64)
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let mut compatible_brands = Vec::new();
        for brand in &self.compatible_brands {
            compatible_brands.push(brand.to_string());
        }
        let s = format!(
            "major_brand={} minor_version={} compatible_brands={}",
            self.major_brand,
            self.minor_version,
            compatible_brands.join("-")
        );
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for FtypAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let major = reader.read_u32::<BigEndian>()?;
        let minor = reader.read_u32::<BigEndian>()?;
        if size % 4 != 0 {
            return Err(Error::InvalidData("invalid ftyp size"));
        }
        let brand_count = (size - 16) / 4; // header + major + minor

        let mut brands = Vec::new();
        for _ in 0..brand_count {
            let b = reader.read_u32::<BigEndian>()?;
            brands.push(From::from(b));
        }

        skip_bytes_to(reader, start + size)?;

        Ok(Self {
            major_brand: From::from(major),
            minor_version: minor,
            compatible_brands: brands,
        })
    }
}

impl<W: Write> WriteAtom<&mut W> for FtypAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        writer.write_u32::<BigEndian>((&self.major_brand).into())?;
        writer.write_u32::<BigEndian>(self.minor_version)?;
        for b in &self.compatible_brands {
            writer.write_u32::<BigEndian>(b.into())?;
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
    fn test_ftyp() {
        let src_box = FtypAtom {
            major_brand: str::parse("isom").unwrap(),
            minor_version: 0,
            compatible_brands: vec![
                str::parse("isom").unwrap(),
                str::parse("iso2").unwrap(),
                str::parse("avc1").unwrap(),
                str::parse("mp41").unwrap(),
            ],
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, FtypAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = FtypAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
