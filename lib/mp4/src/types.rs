use serde::Serialize;
use std::convert::TryFrom;
use std::fmt;

pub use bytes::Bytes;
pub use num_rational::Ratio;

use crate::{Error, Result};

#[derive(Default, Eq, PartialEq, Clone, Copy, Serialize)]
pub struct FourCC(u32);

impl std::str::FromStr for FourCC {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let [a, b, c, d] = s.as_bytes() {
            Ok([*a, *b, *c, *d].into())
        } else {
            Err(Error::InvalidData("expected exactly four bytes in string"))
        }
    }
}

impl FourCC {
    pub const fn new(val: &[u8; 4]) -> Self {
        Self(u32::from_be_bytes(*val))
    }
}

impl From<u32> for FourCC {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<FourCC> for u32 {
    fn from(fourcc: FourCC) -> Self {
        fourcc.0
    }
}

impl From<&FourCC> for u32 {
    fn from(fourcc: &FourCC) -> Self {
        fourcc.0
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(value: [u8; 4]) -> Self {
        Self::new(&value)
    }
}

impl From<&[u8; 4]> for FourCC {
    fn from(value: &[u8; 4]) -> Self {
        Self::new(value)
    }
}

impl From<FourCC> for [u8; 4] {
    fn from(value: FourCC) -> Self {
        value.0.to_be_bytes()
    }
}

impl From<&FourCC> for [u8; 4] {
    fn from(value: &FourCC) -> Self {
        value.0.to_be_bytes()
    }
}

impl fmt::Debug for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code: u32 = self.into();
        let bytes: [u8; 4] = self.into();
        let string = String::from_utf8_lossy(&bytes[..]);
        write!(f, "{} / {:#08X}", string, code)
    }
}

impl fmt::Display for FourCC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes: [u8; 4] = self.into();
        write!(f, "{}", String::from_utf8_lossy(&bytes[..]))
    }
}

const DISPLAY_TYPE_VIDEO: &str = "Video";
const DISPLAY_TYPE_AUDIO: &str = "Audio";
const DISPLAY_TYPE_SUBTITLE: &str = "Subtitle";

const HANDLER_TYPE_VIDEO: &str = "vide";
const HANDLER_TYPE_VIDEO_FOURCC: FourCC = FourCC::new(b"vide");

const HANDLER_TYPE_AUDIO: &str = "soun";
const HANDLER_TYPE_AUDIO_FOURCC: FourCC = FourCC::new(b"soun");

const HANDLER_TYPE_SUBTITLE: &str = "sbtl";
const HANDLER_TYPE_SUBTITLE_FOURCC: FourCC = FourCC::new(b"stbl");

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackType {
    Video,
    Audio,
    Subtitle,
}

impl fmt::Display for TrackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TrackType::Video => DISPLAY_TYPE_VIDEO,
            TrackType::Audio => DISPLAY_TYPE_AUDIO,
            TrackType::Subtitle => DISPLAY_TYPE_SUBTITLE,
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for TrackType {
    type Error = Error;
    fn try_from(handler: &str) -> Result<Self> {
        match handler {
            HANDLER_TYPE_VIDEO => Ok(Self::Video),
            HANDLER_TYPE_AUDIO => Ok(Self::Audio),
            HANDLER_TYPE_SUBTITLE => Ok(Self::Subtitle),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl TryFrom<&FourCC> for TrackType {
    type Error = Error;
    fn try_from(fourcc: &FourCC) -> Result<Self> {
        match *fourcc {
            HANDLER_TYPE_VIDEO_FOURCC => Ok(Self::Video),
            HANDLER_TYPE_AUDIO_FOURCC => Ok(Self::Audio),
            HANDLER_TYPE_SUBTITLE_FOURCC => Ok(Self::Subtitle),
            _ => Err(Error::InvalidData("unsupported handler type")),
        }
    }
}

impl From<TrackType> for FourCC {
    fn from(val: TrackType) -> Self {
        match val {
            TrackType::Video => HANDLER_TYPE_VIDEO_FOURCC,
            TrackType::Audio => HANDLER_TYPE_AUDIO_FOURCC,
            TrackType::Subtitle => HANDLER_TYPE_SUBTITLE_FOURCC,
        }
    }
}

impl TrackType {
    pub fn friendly_name(&self) -> &'static str {
        match self {
            TrackType::Video => "VideoHandler",
            TrackType::Audio => "SoundHandler",
            TrackType::Subtitle => "",
        }
    }
}

const MEDIA_TYPE_H264: &str = "h264";
const MEDIA_TYPE_H265: &str = "h265";
const MEDIA_TYPE_VP9: &str = "vp9";
const MEDIA_TYPE_AAC: &str = "aac";
const MEDIA_TYPE_TTXT: &str = "ttxt";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaType {
    H264,
    H265,
    VP9,
    AAC,
    TTXT,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &str = self.into();
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for MediaType {
    type Error = Error;
    fn try_from(media: &str) -> Result<Self> {
        match media {
            MEDIA_TYPE_H264 => Ok(Self::H264),
            MEDIA_TYPE_H265 => Ok(Self::H265),
            MEDIA_TYPE_VP9 => Ok(Self::VP9),
            MEDIA_TYPE_AAC => Ok(Self::AAC),
            MEDIA_TYPE_TTXT => Ok(Self::TTXT),
            _ => Err(Error::InvalidData("unsupported media type")),
        }
    }
}

impl From<MediaType> for &str {
    fn from(val: MediaType) -> Self {
        match val {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::H265 => MEDIA_TYPE_H265,
            MediaType::VP9 => MEDIA_TYPE_VP9,
            MediaType::AAC => MEDIA_TYPE_AAC,
            MediaType::TTXT => MEDIA_TYPE_TTXT,
        }
    }
}

impl From<&MediaType> for &str {
    fn from(val: &MediaType) -> Self {
        match val {
            MediaType::H264 => MEDIA_TYPE_H264,
            MediaType::H265 => MEDIA_TYPE_H265,
            MediaType::VP9 => MEDIA_TYPE_VP9,
            MediaType::AAC => MEDIA_TYPE_AAC,
            MediaType::TTXT => MEDIA_TYPE_TTXT,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AvcProfile {
    AvcConstrainedBaseline, // 66 with constraint set 1
    AvcBaseline,            // 66,
    AvcMain,                // 77,
    AvcExtended,            // 88,
    AvcHigh,                // 100
                            // TODO Progressive High Profile, Constrained High Profile, ...
}

impl TryFrom<(u8, u8)> for AvcProfile {
    type Error = Error;
    fn try_from(value: (u8, u8)) -> Result<Self> {
        let profile = value.0;
        let constraint_set1_flag = value.1 & 0x40 >> 6;
        match (profile, constraint_set1_flag) {
            (66, 1) => Ok(Self::AvcConstrainedBaseline),
            (66, 0) => Ok(Self::AvcBaseline),
            (77, _) => Ok(Self::AvcMain),
            (88, _) => Ok(Self::AvcExtended),
            (100, _) => Ok(Self::AvcHigh),
            _ => Err(Error::InvalidData("unsupported avc profile")),
        }
    }
}

impl fmt::Display for AvcProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let profile = match self {
            AvcProfile::AvcConstrainedBaseline => "Constrained Baseline",
            AvcProfile::AvcBaseline => "Baseline",
            AvcProfile::AvcMain => "Main",
            AvcProfile::AvcExtended => "Extended",
            AvcProfile::AvcHigh => "High",
        };
        write!(f, "{}", profile)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AudioObjectType {
    AacMain = 1,                                       // AAC Main Profile
    AacLowComplexity = 2,                              // AAC Low Complexity
    AacScalableSampleRate = 3,                         // AAC Scalable Sample Rate
    AacLongTermPrediction = 4,                         // AAC Long Term Predictor
    SpectralBandReplication = 5,                       // Spectral band Replication
    AACScalable = 6,                                   // AAC Scalable
    TwinVQ = 7,                                        // Twin VQ
    CodeExcitedLinearPrediction = 8,                   // CELP
    HarmonicVectorExcitationCoding = 9,                // HVXC
    TextToSpeechtInterface = 12,                       // TTSI
    MainSynthetic = 13,                                // Main Synthetic
    WavetableSynthesis = 14,                           // Wavetable Synthesis
    GeneralMIDI = 15,                                  // General MIDI
    AlgorithmicSynthesis = 16,                         // Algorithmic Synthesis
    ErrorResilientAacLowComplexity = 17,               // ER AAC LC
    ErrorResilientAacLongTermPrediction = 19,          // ER AAC LTP
    ErrorResilientAacScalable = 20,                    // ER AAC Scalable
    ErrorResilientAacTwinVQ = 21,                      // ER AAC TwinVQ
    ErrorResilientAacBitSlicedArithmeticCoding = 22,   // ER Bit Sliced Arithmetic Coding
    ErrorResilientAacLowDelay = 23,                    // ER AAC Low Delay
    ErrorResilientCodeExcitedLinearPrediction = 24,    // ER CELP
    ErrorResilientHarmonicVectorExcitationCoding = 25, // ER HVXC
    ErrorResilientHarmonicIndividualLinesNoise = 26,   // ER HILN
    ErrorResilientParametric = 27,                     // ER Parametric
    SinuSoidalCoding = 28,                             // SSC
    ParametricStereo = 29,                             // PS
    MpegSurround = 30,                                 // MPEG Surround
    MpegLayer1 = 32,                                   // MPEG Layer 1
    MpegLayer2 = 33,                                   // MPEG Layer 2
    MpegLayer3 = 34,                                   // MPEG Layer 3
    DirectStreamTransfer = 35,                         // DST Direct Stream Transfer
    AudioLosslessCoding = 36,                          // ALS Audio Lossless Coding
    ScalableLosslessCoding = 37,                       // SLC Scalable Lossless Coding
    ScalableLosslessCodingNoneCore = 38,               // SLC non-core
    ErrorResilientAacEnhancedLowDelay = 39,            // ER AAC ELD
    SymbolicMusicRepresentationSimple = 40,            // SMR Simple
    SymbolicMusicRepresentationMain = 41,              // SMR Main
    UnifiedSpeechAudioCoding = 42,                     // USAC
    SpatialAudioObjectCoding = 43,                     // SAOC
    LowDelayMpegSurround = 44,                         // LD MPEG Surround
    SpatialAudioObjectCodingDialogueEnhancement = 45,  // SAOC-DE
    AudioSync = 46,                                    // Audio Sync
}

impl TryFrom<u8> for AudioObjectType {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::AacMain),
            2 => Ok(Self::AacLowComplexity),
            3 => Ok(Self::AacScalableSampleRate),
            4 => Ok(Self::AacLongTermPrediction),
            5 => Ok(Self::SpectralBandReplication),
            6 => Ok(Self::AACScalable),
            7 => Ok(Self::TwinVQ),
            8 => Ok(Self::CodeExcitedLinearPrediction),
            9 => Ok(Self::HarmonicVectorExcitationCoding),
            12 => Ok(Self::TextToSpeechtInterface),
            13 => Ok(Self::MainSynthetic),
            14 => Ok(Self::WavetableSynthesis),
            15 => Ok(Self::GeneralMIDI),
            16 => Ok(Self::AlgorithmicSynthesis),
            17 => Ok(Self::ErrorResilientAacLowComplexity),
            19 => Ok(Self::ErrorResilientAacLongTermPrediction),
            20 => Ok(Self::ErrorResilientAacScalable),
            21 => Ok(Self::ErrorResilientAacTwinVQ),
            22 => Ok(Self::ErrorResilientAacBitSlicedArithmeticCoding),
            23 => Ok(Self::ErrorResilientAacLowDelay),
            24 => Ok(Self::ErrorResilientCodeExcitedLinearPrediction),
            25 => Ok(Self::ErrorResilientHarmonicVectorExcitationCoding),
            26 => Ok(Self::ErrorResilientHarmonicIndividualLinesNoise),
            27 => Ok(Self::ErrorResilientParametric),
            28 => Ok(Self::SinuSoidalCoding),
            29 => Ok(Self::ParametricStereo),
            30 => Ok(Self::MpegSurround),
            32 => Ok(Self::MpegLayer1),
            33 => Ok(Self::MpegLayer2),
            34 => Ok(Self::MpegLayer3),
            35 => Ok(Self::DirectStreamTransfer),
            36 => Ok(Self::AudioLosslessCoding),
            37 => Ok(Self::ScalableLosslessCoding),
            38 => Ok(Self::ScalableLosslessCodingNoneCore),
            39 => Ok(Self::ErrorResilientAacEnhancedLowDelay),
            40 => Ok(Self::SymbolicMusicRepresentationSimple),
            41 => Ok(Self::SymbolicMusicRepresentationMain),
            42 => Ok(Self::UnifiedSpeechAudioCoding),
            43 => Ok(Self::SpatialAudioObjectCoding),
            44 => Ok(Self::LowDelayMpegSurround),
            45 => Ok(Self::SpatialAudioObjectCodingDialogueEnhancement),
            46 => Ok(Self::AudioSync),
            _ => Err(Error::InvalidData("invalid audio object type")),
        }
    }
}

impl fmt::Display for AudioObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            AudioObjectType::AacMain => "AAC Main",
            AudioObjectType::AacLowComplexity => "LC",
            AudioObjectType::AacScalableSampleRate => "SSR",
            AudioObjectType::AacLongTermPrediction => "LTP",
            AudioObjectType::SpectralBandReplication => "SBR",
            AudioObjectType::AACScalable => "Scalable",
            AudioObjectType::TwinVQ => "TwinVQ",
            AudioObjectType::CodeExcitedLinearPrediction => "CELP",
            AudioObjectType::HarmonicVectorExcitationCoding => "HVXC",
            AudioObjectType::TextToSpeechtInterface => "TTSI",
            AudioObjectType::MainSynthetic => "Main Synthetic",
            AudioObjectType::WavetableSynthesis => "Wavetable Synthesis",
            AudioObjectType::GeneralMIDI => "General MIDI",
            AudioObjectType::AlgorithmicSynthesis => "Algorithmic Synthesis",
            AudioObjectType::ErrorResilientAacLowComplexity => "ER AAC LC",
            AudioObjectType::ErrorResilientAacLongTermPrediction => "ER AAC LTP",
            AudioObjectType::ErrorResilientAacScalable => "ER AAC scalable",
            AudioObjectType::ErrorResilientAacTwinVQ => "ER AAC TwinVQ",
            AudioObjectType::ErrorResilientAacBitSlicedArithmeticCoding => "ER AAC BSAC",
            AudioObjectType::ErrorResilientAacLowDelay => "ER AAC LD",
            AudioObjectType::ErrorResilientCodeExcitedLinearPrediction => "ER CELP",
            AudioObjectType::ErrorResilientHarmonicVectorExcitationCoding => "ER HVXC",
            AudioObjectType::ErrorResilientHarmonicIndividualLinesNoise => "ER HILN",
            AudioObjectType::ErrorResilientParametric => "ER Parametric",
            AudioObjectType::SinuSoidalCoding => "SSC",
            AudioObjectType::ParametricStereo => "Parametric Stereo",
            AudioObjectType::MpegSurround => "MPEG surround",
            AudioObjectType::MpegLayer1 => "MPEG Layer 1",
            AudioObjectType::MpegLayer2 => "MPEG Layer 2",
            AudioObjectType::MpegLayer3 => "MPEG Layer 3",
            AudioObjectType::DirectStreamTransfer => "DST",
            AudioObjectType::AudioLosslessCoding => "ALS",
            AudioObjectType::ScalableLosslessCoding => "SLS",
            AudioObjectType::ScalableLosslessCodingNoneCore => "SLS Non-core",
            AudioObjectType::ErrorResilientAacEnhancedLowDelay => "ER AAC ELD",
            AudioObjectType::SymbolicMusicRepresentationSimple => "SMR Simple",
            AudioObjectType::SymbolicMusicRepresentationMain => "SMR Main",
            AudioObjectType::UnifiedSpeechAudioCoding => "USAC",
            AudioObjectType::SpatialAudioObjectCoding => "SAOC",
            AudioObjectType::LowDelayMpegSurround => "LD MPEG Surround",
            AudioObjectType::SpatialAudioObjectCodingDialogueEnhancement => "SAOC-DE",
            AudioObjectType::AudioSync => "Audio Sync",
        };
        write!(f, "{}", type_str)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SampleFreqIndex {
    Freq96000 = 0x0,
    Freq88200 = 0x1,
    Freq64000 = 0x2,
    Freq48000 = 0x3,
    Freq44100 = 0x4,
    Freq32000 = 0x5,
    Freq24000 = 0x6,
    Freq22050 = 0x7,
    Freq16000 = 0x8,
    Freq12000 = 0x9,
    Freq11025 = 0xa,
    Freq8000 = 0xb,
    Freq7350 = 0xc,
}

impl TryFrom<u8> for SampleFreqIndex {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x0 => Ok(Self::Freq96000),
            0x1 => Ok(Self::Freq88200),
            0x2 => Ok(Self::Freq64000),
            0x3 => Ok(Self::Freq48000),
            0x4 => Ok(Self::Freq44100),
            0x5 => Ok(Self::Freq32000),
            0x6 => Ok(Self::Freq24000),
            0x7 => Ok(Self::Freq22050),
            0x8 => Ok(Self::Freq16000),
            0x9 => Ok(Self::Freq12000),
            0xa => Ok(Self::Freq11025),
            0xb => Ok(Self::Freq8000),
            0xc => Ok(Self::Freq7350),
            _ => Err(Error::InvalidData("invalid sampling frequency index")),
        }
    }
}

impl SampleFreqIndex {
    pub fn freq(&self) -> u32 {
        match self {
            SampleFreqIndex::Freq96000 => 96000,
            SampleFreqIndex::Freq88200 => 88200,
            SampleFreqIndex::Freq64000 => 64000,
            SampleFreqIndex::Freq48000 => 48000,
            SampleFreqIndex::Freq44100 => 44100,
            SampleFreqIndex::Freq32000 => 32000,
            SampleFreqIndex::Freq24000 => 24000,
            SampleFreqIndex::Freq22050 => 22050,
            SampleFreqIndex::Freq16000 => 16000,
            SampleFreqIndex::Freq12000 => 12000,
            SampleFreqIndex::Freq11025 => 11025,
            SampleFreqIndex::Freq8000 => 8000,
            SampleFreqIndex::Freq7350 => 7350,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChannelConfig {
    Mono = 0x1,
    Stereo = 0x2,
    Three = 0x3,
    Four = 0x4,
    Five = 0x5,
    FiveOne = 0x6,
    SevenOne = 0x7,
}

impl TryFrom<u8> for ChannelConfig {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x1 => Ok(Self::Mono),
            0x2 => Ok(Self::Stereo),
            0x3 => Ok(Self::Three),
            0x4 => Ok(Self::Four),
            0x5 => Ok(Self::Five),
            0x6 => Ok(Self::FiveOne),
            0x7 => Ok(Self::SevenOne),
            _ => Err(Error::InvalidData("invalid channel configuration")),
        }
    }
}

impl fmt::Display for ChannelConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ChannelConfig::Mono => "mono",
            ChannelConfig::Stereo => "stereo",
            ChannelConfig::Three => "three",
            ChannelConfig::Four => "four",
            ChannelConfig::Five => "five",
            ChannelConfig::FiveOne => "five.one",
            ChannelConfig::SevenOne => "seven.one",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AvcConfig {
    pub width: u16,
    pub height: u16,
    pub seq_param_set: Vec<u8>,
    pub pic_param_set: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct HevcConfig {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Vp9Config {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AacConfig {
    pub bitrate: u32,
    pub profile: AudioObjectType,
    pub freq_index: SampleFreqIndex,
    pub chan_conf: ChannelConfig,
}

impl Default for AacConfig {
    fn default() -> Self {
        Self {
            bitrate: 0,
            profile: AudioObjectType::AacLowComplexity,
            freq_index: SampleFreqIndex::Freq48000,
            chan_conf: ChannelConfig::Stereo,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TtxtConfig {}

#[derive(Debug, PartialEq, Clone)]
pub enum MediaConfig {
    AvcConfig(AvcConfig),
    HevcConfig(HevcConfig),
    Vp9Config(Vp9Config),
    AacConfig(AacConfig),
    TtxtConfig(TtxtConfig),
}

#[derive(Debug)]
pub struct Mp4Sample {
    pub start_time: u64,
    pub duration: u32,
    pub rendering_offset: i32,
    pub is_sync: bool,
    pub bytes: Bytes,
}

impl PartialEq for Mp4Sample {
    fn eq(&self, other: &Self) -> bool {
        self.start_time == other.start_time
            && self.duration == other.duration
            && self.rendering_offset == other.rendering_offset
            && self.is_sync == other.is_sync
            && self.bytes.len() == other.bytes.len() // XXX for easy check
    }
}

impl fmt::Display for Mp4Sample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start_time {}, duration {}, rendering_offset {}, is_sync {}, length {}",
            self.start_time,
            self.duration,
            self.rendering_offset,
            self.is_sync,
            self.bytes.len()
        )
    }
}

pub fn creation_time(creation_time: u64) -> u64 {
    // convert from MP4 epoch (1904-01-01) to Unix epoch (1970-01-01)
    if creation_time >= 2082844800 {
        creation_time - 2082844800
    } else {
        creation_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fourcc() {
        let ftyp_fcc = 0x66747970;
        assert_eq!(FourCC::from(b"ftyp").0, ftyp_fcc);
    }
}
