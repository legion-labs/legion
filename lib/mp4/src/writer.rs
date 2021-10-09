use std::io::Write;

use crate::atoms::avc1::Avc1Atom;
use crate::atoms::ftyp::FtypAtom;
use crate::atoms::hev1::Hev1Atom;
use crate::atoms::moov::MoovAtom;
use crate::atoms::mp4a::Mp4aAtom;
use crate::atoms::smhd::SmhdAtom;
use crate::atoms::stco::StcoAtom;
use crate::atoms::stss::StssAtom;
use crate::atoms::trak::TrakAtom;
use crate::atoms::tx3g::Tx3gAtom;
use crate::atoms::vmhd::VmhdAtom;
use crate::atoms::vp09::Vp09Atom;
use crate::atoms::WriteAtom;
use crate::{AacConfig, AvcConfig, HevcConfig, MediaConfig, TrackType, TtxtConfig, Vp9Config};
use crate::{FourCC, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Mp4Config {
    pub major_brand: FourCC,
    pub minor_version: u32,
    pub compatible_brands: Vec<FourCC>,
    pub timescale: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackConfig {
    pub track_type: TrackType,
    pub timescale: u32,
    pub language: String,
    pub media_conf: MediaConfig,
}

impl From<MediaConfig> for TrackConfig {
    fn from(media_conf: MediaConfig) -> Self {
        match media_conf {
            MediaConfig::AvcConfig(avc_conf) => Self::from(avc_conf),
            MediaConfig::HevcConfig(hevc_conf) => Self::from(hevc_conf),
            MediaConfig::AacConfig(aac_conf) => Self::from(aac_conf),
            MediaConfig::TtxtConfig(ttxt_conf) => Self::from(ttxt_conf),
            MediaConfig::Vp9Config(vp9_config) => Self::from(vp9_config),
        }
    }
}

impl From<AvcConfig> for TrackConfig {
    fn from(avc_conf: AvcConfig) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 90_000,             // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::AvcConfig(avc_conf),
        }
    }
}

impl From<HevcConfig> for TrackConfig {
    fn from(hevc_conf: HevcConfig) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 90_000,             // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::HevcConfig(hevc_conf),
        }
    }
}

impl From<AacConfig> for TrackConfig {
    fn from(aac_conf: AacConfig) -> Self {
        Self {
            track_type: TrackType::Audio,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::AacConfig(aac_conf),
        }
    }
}

impl From<TtxtConfig> for TrackConfig {
    fn from(txtt_conf: TtxtConfig) -> Self {
        Self {
            track_type: TrackType::Subtitle,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::TtxtConfig(txtt_conf),
        }
    }
}

impl From<Vp9Config> for TrackConfig {
    fn from(vp9_conf: Vp9Config) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 90_000,             // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Vp9Config(vp9_conf),
        }
    }
}

#[derive(Debug)]
pub struct StreamWriter<W> {
    writer: W,
    cur_offset: u64,
    timescale: u32,
}

impl<W> StreamWriter<W> {
    /// Consume self, returning the inner writer.
    ///
    /// This can be useful to recover the inner writer after completion in case
    /// it's owned by the [`Mp4Writer`] instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use legion_mp4::{StreamWriter, Mp4Config};
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
    /// let data = Cursor::new(Vec::<u8>::new());
    /// let mut writer = StreamWriter::write_start(data, &config)?;
    ///
    /// let data: Vec<u8> = writer.into_writer().into_inner();
    /// # Ok(()) }
    /// ```
    pub fn into_writer(self) -> W {
        self.writer
    }
}

impl<W: Write> StreamWriter<W> {
    pub fn write_start(mut writer: W, mp4_config: &Mp4Config) -> Result<Self> {
        let ftyp = FtypAtom {
            major_brand: mp4_config.major_brand,
            minor_version: mp4_config.minor_version,
            compatible_brands: mp4_config.compatible_brands.clone(),
        };
        let cur_offset = ftyp.write_atom(&mut writer)?;

        Ok(Self {
            writer,
            cur_offset,
            timescale: mp4_config.timescale,
        })
    }

    pub fn write_index(&mut self, config: &TrackConfig) -> Result<()> {
        let mut trak = TrakAtom::default();
        trak.tkhd.track_id = 1;
        trak.mdia.mdhd.timescale = config.timescale;
        trak.mdia.mdhd.language = config.language.clone();
        trak.mdia.hdlr.handler_type = config.track_type.into();
        // XXX largesize
        trak.mdia.minf.stbl.stco = Some(StcoAtom::default());
        match config.media_conf {
            MediaConfig::AvcConfig(ref avc_config) => {
                trak.tkhd.set_width(avc_config.width);
                trak.tkhd.set_height(avc_config.height);

                let vmhd = VmhdAtom::default();
                trak.mdia.minf.vmhd = Some(vmhd);

                let avc1 = Avc1Atom::new(avc_config);
                trak.mdia.minf.stbl.stsd.avc1 = Some(avc1);
                trak.mdia.minf.stbl.stss = Some(StssAtom::default());
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
        moov.write_atom(&mut self.writer)?;
        Ok(())
    }

    //pub fn write_sample(&mut self, _key_frame: bool, _content: &[u8]) -> Result<()> {
    //    Ok(())
    //}
}
