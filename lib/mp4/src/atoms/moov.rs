use serde::Serialize;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{Error, FourCC, Result};

use super::mvex::MvexAtom;
use super::mvhd::MvhdAtom;
use super::trak::TrakAtom;
use super::{
    box_start, skip_atom, skip_bytes_to, Atom, AtomHeader, ReadAtom, WriteAtom, HEADER_SIZE,
};

/// Movie Atom
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct MoovAtom {
    pub mvhd: MvhdAtom,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mvex: Option<MvexAtom>,

    #[serde(rename = "trak")]
    pub traks: Vec<TrakAtom>,
}

impl Atom for MoovAtom {
    const FOUR_CC: FourCC = FourCC::new(b"moov");

    fn size(&self) -> u64 {
        let mut size = HEADER_SIZE + self.mvhd.size();
        for trak in &self.traks {
            size += trak.size();
        }
        if let Some(mvex) = &self.mvex {
            size += mvex.size();
        }
        size
    }

    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self).unwrap())
    }

    fn summary(&self) -> Result<String> {
        let s = format!("traks={}", self.traks.len());
        Ok(s)
    }
}

impl<R: Read + Seek> ReadAtom<&mut R> for MoovAtom {
    fn read_atom(reader: &mut R, size: u64) -> Result<Self> {
        let start = box_start(reader)?;

        let mut mvhd = None;
        let mut mvex = None;
        let mut traks = Vec::new();

        let mut current = reader.seek(SeekFrom::Current(0))?;
        let end = start + size;
        while current < end {
            // Get box header.
            let header = AtomHeader::read(reader)?;
            let AtomHeader { name, size: s } = header;

            match name {
                MvhdAtom::FOUR_CC => {
                    mvhd = Some(MvhdAtom::read_atom(reader, s)?);
                }
                MvexAtom::FOUR_CC => {
                    mvex = Some(MvexAtom::read_atom(reader, s)?);
                }
                TrakAtom::FOUR_CC => {
                    let trak = TrakAtom::read_atom(reader, s)?;
                    traks.push(trak);
                }
                //UdtaBox::FOUR_CC => {
                //    // XXX warn!()
                //    skip_box(reader, s)?;
                //}
                _ => {
                    // XXX warn!()
                    skip_atom(reader, s)?;
                }
            }

            current = reader.seek(SeekFrom::Current(0))?;
        }

        if mvhd.is_none() {
            return Err(Error::BoxNotFound(MvhdAtom::FOUR_CC));
        }
        let mvhd = mvhd.unwrap();

        skip_bytes_to(reader, start + size)?;

        Ok(Self { mvhd, mvex, traks })
    }
}

impl<W: Write> WriteAtom<&mut W> for MoovAtom {
    fn write_atom(&self, writer: &mut W) -> Result<u64> {
        AtomHeader::new(self).write(writer)?;

        self.mvhd.write_atom(writer)?;

        for trak in &self.traks {
            trak.write_atom(writer)?;
        }

        if let Some(mvex) = &self.mvex {
            mvex.write_atom(writer)?;
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atoms::avc1::Avc1Atom;
    use crate::atoms::mehd::MehdAtom;
    use crate::atoms::stco::StcoAtom;
    use crate::atoms::trex::TrexAtom;
    use crate::atoms::vmhd::VmhdAtom;
    use crate::atoms::{AtomHeader, SampleFlags};
    use std::io::Cursor;

    #[test]
    fn test_moov() {
        let mut trak = TrakAtom::default();
        trak.tkhd.track_id = 1;
        trak.mdia.mdhd.timescale = 1000;
        trak.mdia.mdhd.language = "und".into();
        trak.mdia.hdlr.handler_type = b"vide".into();
        trak.mdia.hdlr.name = "VideoHandler".into();

        trak.mdia.minf.stbl.stco = Some(StcoAtom::default());
        trak.tkhd.set_width(0);
        trak.tkhd.set_height(0);

        let vmhd = VmhdAtom::default();
        trak.mdia.minf.vmhd = Some(vmhd);

        let avc1 = Avc1Atom::default();
        trak.mdia.minf.stbl.stsd.avc1 = Some(avc1);

        let mut src_box = MoovAtom::default();

        src_box.traks.push(trak);

        src_box.mvhd.timescale = 1000;
        src_box.mvhd.duration = 0;
        src_box.mvhd.next_track_id = 2;

        // fragmentation enabled only
        src_box.mvex = Some(MvexAtom {
            mehd: Some(MehdAtom::default()),
            trex: TrexAtom {
                version: 0,
                flags: 0,
                track_id: 1,
                default_sample_description_index: 1,
                default_sample_duration: 0,
                default_sample_size: 0,
                default_sample_flags: SampleFlags::default(),
            },
        });

        let mut buf = Vec::new();
        src_box.write_atom(&mut buf).unwrap();
        assert_eq!(buf.len(), src_box.size() as usize);

        let mut reader = Cursor::new(&buf);
        let header = AtomHeader::read(&mut reader).unwrap();
        assert_eq!(header.name, MoovAtom::FOUR_CC);
        assert_eq!(src_box.size(), header.size);

        let dst_box = MoovAtom::read_atom(&mut reader, header.size).unwrap();
        assert_eq!(src_box, dst_box);
    }
}
