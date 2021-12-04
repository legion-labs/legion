use std::io::Write;

use crate::atoms::avc1::Avc1Atom;
use crate::atoms::ftyp::FtypAtom;
use crate::atoms::hev1::Hev1Atom;
use crate::atoms::mdat::MdatAtom;
use crate::atoms::mehd::MehdAtom;
use crate::atoms::mfhd::MfhdAtom;
use crate::atoms::moof::MoofAtom;
use crate::atoms::moov::MoovAtom;
use crate::atoms::mp4a::Mp4aAtom;
use crate::atoms::mvex::MvexAtom;
use crate::atoms::smhd::SmhdAtom;
use crate::atoms::stco::StcoAtom;
use crate::atoms::tfdt::TfdtAtom;
use crate::atoms::tfhd::TfhdAtom;
use crate::atoms::traf::TrafAtom;
use crate::atoms::trak::TrakAtom;
use crate::atoms::trex::TrexAtom;
use crate::atoms::trun::TrunAtom;
use crate::atoms::tx3g::Tx3gAtom;
use crate::atoms::vmhd::VmhdAtom;
use crate::atoms::vp09::Vp09Atom;
use crate::atoms::{Atom, SampleDependsOn, SampleFlags, WriteAtom};
use crate::{MediaConfig, Mp4Config, Result, TrackConfig};

/// This writer provides an MSE compatible Byte Stream as described
/// [here](https://w3c.github.io/mse-byte-stream-format-isobmff/)
/// can be used with [MSE Extension](https://github.com/w3c/media-source)
/// Consume self, returning the inner writer.
///
/// This can be useful to recover the inner writer after completion in case
/// it's owned by the [`Mp4Stream`] instance.
///
/// # Examples
///
/// ```rust
/// use legion_mp4::{Mp4Stream, Mp4Config};
/// use std::io::Cursor;
///
/// # fn main() -> legion_mp4::Result<()> {
/// let config = Mp4Config {
///     major_brand: str::parse("isom").unwrap(),
///     minor_version: 512,
///     compatible_brands: vec![
///         str::parse("isom").unwrap(),
///         str::parse("iso2").unwrap(),
///         str::parse("avc1").unwrap(),
///         str::parse("mp41").unwrap(),
///     ],
///     timescale: 1000,
/// };
///
/// let mut data = Cursor::new(Vec::<u8>::new());
/// Mp4Stream::write_start(&config, 30, &mut data)?;
///
/// let data: Vec<u8> = data.into_inner();
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct Mp4Stream {
    timescale: u32,
    fps: u32,
    moof: MoofAtom,
}

impl Mp4Stream {
    /// # Errors
    pub fn write_start<W: Write>(mp4_config: &Mp4Config, fps: u32, writer: &mut W) -> Result<Self> {
        let ftyp = FtypAtom {
            major_brand: mp4_config.major_brand,
            minor_version: mp4_config.minor_version,
            compatible_brands: mp4_config.compatible_brands.clone(),
        };
        ftyp.write_atom(writer)?;
        let moof = MoofAtom {
            mfhd: MfhdAtom {
                sequence_number: 0,
                ..MfhdAtom::default()
            },
            trafs: vec![TrafAtom {
                tfhd: TfhdAtom {
                    track_id: 1,
                    default_sample_flags: Some(SampleFlags {
                        sample_is_non_sync_sample: true,
                        sample_depends_on: SampleDependsOn::Others,
                        ..SampleFlags::default()
                    }),
                    default_base_is_moof: true,
                    ..TfhdAtom::default()
                },
                trun: Some(TrunAtom {
                    sample_count: 1,
                    data_offset: Some(0),
                    first_sample_flags: Some(SampleFlags {
                        sample_depends_on: SampleDependsOn::None,
                        ..SampleFlags::default()
                    }),
                    sample_durations: Some(vec![0]),
                    sample_sizes: Some(vec![0]),
                    ..TrunAtom::default()
                }),
                tfdt: Some(TfdtAtom {
                    version: 1,
                    flags: 0,
                    decode_time: 0,
                }),
            }],
        };
        Ok(Self {
            timescale: mp4_config.timescale,
            fps,
            moof,
        })
    }

    /// # Errors
    pub fn write_index<W: Write>(&mut self, config: &TrackConfig, writer: &mut W) -> Result<()> {
        let mut trak = TrakAtom::default();
        trak.tkhd.track_id = 1;
        trak.mdia.mdhd.timescale = config.timescale;
        trak.mdia.mdhd.language = config.language.clone();
        trak.mdia.hdlr.handler_type = config.track_type.into();
        trak.mdia.hdlr.name = config.track_type.friendly_name().into();

        // XXX largesize
        trak.mdia.minf.stbl.stco = Some(StcoAtom::default());
        match config.media_conf {
            MediaConfig::AvcConfig(ref avc_config) => {
                trak.tkhd.set_width(avc_config.width);
                trak.tkhd.set_height(avc_config.height);

                let vmhd = VmhdAtom {
                    ..VmhdAtom::default()
                };
                trak.mdia.minf.vmhd = Some(vmhd);

                let avc1 = Avc1Atom::new(avc_config);
                trak.mdia.minf.stbl.stsd.avc1 = Some(avc1);
            }
            MediaConfig::HevcConfig(ref hevc_config) => {
                trak.tkhd.set_width(hevc_config.width);
                trak.tkhd.set_height(hevc_config.height);

                let vmhd = VmhdAtom::default();
                trak.mdia.minf.vmhd = Some(vmhd);

                let hev1 = Hev1Atom::new(hevc_config);
                trak.mdia.minf.stbl.stsd.hev1 = Some(hev1);
            }
            MediaConfig::Vp9Config(ref config) => {
                trak.tkhd.set_width(config.width);
                trak.tkhd.set_height(config.height);

                trak.mdia.minf.stbl.stsd.vp09 = Some(Vp09Atom::new(config));
            }
            MediaConfig::AacConfig(ref aac_config) => {
                let smhd = SmhdAtom::default();
                trak.mdia.minf.smhd = Some(smhd);

                let mp4a = Mp4aAtom::new(aac_config);
                trak.mdia.minf.stbl.stsd.mp4a = Some(mp4a);
            }
            MediaConfig::TtxtConfig(ref _ttxt_config) => {
                let tx3g = Tx3gAtom::default();
                trak.mdia.minf.stbl.stsd.tx3g = Some(tx3g);
            }
        }
        let mut moov = MoovAtom::default();

        moov.traks.push(trak);

        moov.mvhd.timescale = self.timescale;
        moov.mvhd.duration = 0;
        moov.mvhd.next_track_id = 2;

        // fragmentation enabled only
        moov.mvex = Some(MvexAtom {
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

        moov.write_atom(writer)?;

        Ok(())
    }

    /// # Errors
    pub fn write_sample<W: Write>(
        &mut self,
        key_frame: bool,
        content: &[u8],
        writer: &mut W,
    ) -> Result<()> {
        let duration = 90000 / self.fps;
        let timestamp = self.moof.mfhd.sequence_number * duration;
        self.moof.mfhd.sequence_number += 1;
        self.moof.trafs[0]
            .trun
            .as_mut()
            .unwrap()
            .sample_sizes
            .as_mut()
            .unwrap()[0] = content.len() as u32;
        if key_frame {
            self.moof.trafs[0].trun.as_mut().unwrap().first_sample_flags = Some(SampleFlags {
                sample_depends_on: SampleDependsOn::None,
                ..SampleFlags::default()
            });
        } else {
            self.moof.trafs[0].trun.as_mut().unwrap().first_sample_flags = None;
        }
        let size = self.moof.size() + 8;
        self.moof.trafs[0].trun.as_mut().unwrap().data_offset = Some(size as i32);
        self.moof.trafs[0]
            .trun
            .as_mut()
            .unwrap()
            .sample_durations
            .as_mut()
            .unwrap()[0] = duration;
        self.moof.trafs[0].tfdt.as_mut().unwrap().decode_time = u64::from(timestamp);

        self.moof.write_atom(writer)?;
        let mdat = MdatAtom::Borrowed(content);
        mdat.write_atom(writer)?;
        Ok(())
    }
}
