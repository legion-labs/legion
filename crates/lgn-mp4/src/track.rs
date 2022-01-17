use crate::{AacConfig, AvcConfig, HevcConfig, MediaConfig, TrackType, TtxtConfig, Vp9Config};

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
            MediaConfig::Avc(avc_conf) => Self::from(avc_conf),
            MediaConfig::Hevc(hevc_conf) => Self::from(hevc_conf),
            MediaConfig::Vp9(vp9_config) => Self::from(vp9_config),
            MediaConfig::Aac(aac_conf) => Self::from(aac_conf),
            MediaConfig::Ttxt(ttxt_conf) => Self::from(ttxt_conf),
        }
    }
}

impl From<AvcConfig> for TrackConfig {
    fn from(avc_conf: AvcConfig) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 90000,              // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Avc(avc_conf),
        }
    }
}

impl From<HevcConfig> for TrackConfig {
    fn from(hevc_conf: HevcConfig) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 90000,              // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Hevc(hevc_conf),
        }
    }
}

impl From<Vp9Config> for TrackConfig {
    fn from(vp9_conf: Vp9Config) -> Self {
        Self {
            track_type: TrackType::Video,
            timescale: 90000,              // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Vp9(vp9_conf),
        }
    }
}

impl From<AacConfig> for TrackConfig {
    fn from(aac_conf: AacConfig) -> Self {
        Self {
            track_type: TrackType::Audio,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Aac(aac_conf),
        }
    }
}

impl From<TtxtConfig> for TrackConfig {
    fn from(txtt_conf: TtxtConfig) -> Self {
        Self {
            track_type: TrackType::Subtitle,
            timescale: 1000,               // XXX
            language: String::from("und"), // XXX
            media_conf: MediaConfig::Ttxt(txtt_conf),
        }
    }
}
