use std::io::{Read, Seek, SeekFrom, Write};

use serde::Serialize;

use super::tfdt::TfdtAtom;
use super::tfhd::TfhdAtom;
use super::trun::TrunAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};
use crate::{Error, FourCC, Result};

/// Track Fragment Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct TrafAtom {
    pub tfhd: TfhdAtom,
    pub tfdt: Option<TfdtAtom>,
    pub trun: Option<TrunAtom>,
}

impl Atom for TrafAtom {
    const FOUR_CC: FourCC = FourCC::new(b"traf");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE;
        size += self.tfhd.size();
        if let Some(ref tfdt) = self.tfdt {
            size += tfdt.size();
        }
        if let Some(ref trun) = self.trun {
            size += trun.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for TrafAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut tfhd = None;
        let mut tfdt = None;
        let mut trun = None;

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                TfhdAtom::FOUR_CC => {
                    tfhd = Some(TfhdAtom::read_atom(reader, s)?);
                }
                TrunAtom::FOUR_CC => {
                    trun = Some(TrunAtom::read_atom(reader, s)?);
                }
                TfdtAtom::FOUR_CC => {
                    tfdt = Some(TfdtAtom::read_atom(reader, s)?);
                }
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if tfhd.is_none() {
            return Err(Error::BoxNotFound(TfhdAtom::FOUR_CC));
        }
        let tfhd = tfhd.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { tfhd, tfdt, trun })
    }
}

impl<W: Write> WriteAtom<&mut W> for TrafAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.tfhd.write_atom(writer)?;

        if let Some(tfdt) = &self.tfdt {
            tfdt.write_atom(writer)?;
        }

        if let Some(trun) = &self.trun {
            trun.write_atom(writer)?;
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
    fn test_traf_same_size() {
        let src_box = TrafAtom {
            tfhd: TfhdAtom {
                track_id: 1,
                default_sample_flags: Some(0x1010000.into()),
                default_base_is_moof: true,
                ..TfhdAtom::default()
            },
            trun: Some(TrunAtom {
                version: 0,
                sample_count: 1,
                data_offset: Some(0),
                first_sample_flags: Some(0x2000000.into()),
                sample_durations: Some(vec![0]),
                sample_sizes: Some(vec![0]),
                sample_flags: None,
                sample_cts: None,
            }),
            tfdt: Some(TfdtAtom {
                version: 1,
                flags: 0,
                decode_time: 0,
            }),
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TrafAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TrafAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }

    #[test]
    fn test_traf_without_tfdt() {
        let src_box = TrafAtom {
            tfhd: TfhdAtom {
                track_id: 1,
                default_sample_flags: Some(0x1010000.into()),
                default_base_is_moof: true,
                ..TfhdAtom::default()
            },
            trun: Some(TrunAtom {
                version: 0,
                sample_count: 1,
                data_offset: Some(0),
                first_sample_flags: Some(0x2000000.into()),
                sample_durations: Some(vec![0]),
                sample_sizes: Some(vec![0]),
                sample_flags: None,
                sample_cts: None,
            }),
            tfdt: None,
        };
        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, TrafAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = TrafAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
