use bytes::Bytes;
use num_derive::FromPrimitive;

use super::config::AudioObjectType;

#[derive(Debug, Clone)]
pub enum AacProfile {
    AacProfileReserved = 3,

    // @see 7.1 Profiles, ISO_IEC_13818-7-AAC-2004.pdf, page 40
    AacProfileMain = 0,
    AacProfileLC = 1,
    AacProfileSSR = 2,
}

impl Default for AacProfile {
    fn default() -> Self {
        AacProfile::AacProfileReserved
    }
}

impl From<u8> for AacProfile {
    fn from(u: u8) -> Self {
        match u {
            3u8 => Self::AacProfileReserved,
            0u8 => Self::AacProfileMain,
            1u8 => Self::AacProfileLC,
            2u8 => Self::AacProfileSSR,
            _ => Self::AacProfileReserved,
        }
    }
}

impl AacProfile {
    pub fn from_u8(&self) -> u8 {
        match self {
            AacProfile::AacProfileReserved => 3u8,
            AacProfile::AacProfileMain => 0u8,
            AacProfile::AacProfileLC => 1u8,
            AacProfile::AacProfileSSR => 2u8,
        }
    }
}

impl Into<AudioObjectType> for AacProfile {
    fn into(self) -> AudioObjectType {
        match self {
            Self::AacProfileMain => AudioObjectType::AacMain,
            Self::AacProfileLC => AudioObjectType::AacLowComplexity,
            Self::AacProfileSSR => AudioObjectType::AacScalableSampleRate,
            _ => AudioObjectType::Reserved,
        }
    }
}

#[derive(Clone, Debug, FromPrimitive)]
#[repr(u8)]
/// Sampling Frequency Index
/// ISO/IEC 14496-3:2019(E) - 1.6.2.4 (Table 1.22)
pub enum SampleFrequencyIndex {
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
    Freq11025 = 0xA,
    Freq8000 = 0xB,
    Freq7350 = 0xC,
    FreqReserved = 0xD,
    FreqReserved2 = 0xE,
    FreqEscape = 0xF,
}

impl SampleFrequencyIndex {
    pub fn to_freq(&self) -> u32 {
        match self {
            SampleFrequencyIndex::Freq96000 => 96000,
            SampleFrequencyIndex::Freq88200 => 88200,
            SampleFrequencyIndex::Freq64000 => 64000,
            SampleFrequencyIndex::Freq48000 => 48000,
            SampleFrequencyIndex::Freq44100 => 44100,
            SampleFrequencyIndex::Freq32000 => 32000,
            SampleFrequencyIndex::Freq24000 => 24000,
            SampleFrequencyIndex::Freq22050 => 22050,
            SampleFrequencyIndex::Freq16000 => 16000,
            SampleFrequencyIndex::Freq12000 => 12000,
            SampleFrequencyIndex::Freq11025 => 11025,
            SampleFrequencyIndex::Freq8000 => 8000,
            SampleFrequencyIndex::Freq7350 => 7350,
            SampleFrequencyIndex::FreqReserved => 0,
            SampleFrequencyIndex::FreqReserved2 => 0,
            SampleFrequencyIndex::FreqEscape => 0,
        }
    }
}

impl From<u8> for SampleFrequencyIndex {
    fn from(u: u8) -> Self {
        match u {
            0x0 => Self::Freq96000,
            0x1 => Self::Freq88200,
            0x2 => Self::Freq64000,
            0x3 => Self::Freq48000,
            0x4 => Self::Freq44100,
            0x5 => Self::Freq32000,
            0x6 => Self::Freq24000,
            0x7 => Self::Freq22050,
            0x8 => Self::Freq16000,
            0x9 => Self::Freq12000,
            0xA => Self::Freq11025,
            0xB => Self::Freq8000,
            0xC => Self::Freq7350,
            0xD => Self::FreqReserved,
            0xE => Self::FreqReserved2,
            _ => Self::FreqEscape,
        }
    }
}

impl Default for SampleFrequencyIndex {
    fn default() -> Self {
        SampleFrequencyIndex::FreqReserved
    }
}

#[derive(Debug, Clone)]
pub struct RawAacStreamCodec {
    // Codec level informations.
    pub protection_absent: u8,
    pub aac_object: AudioObjectType,
    pub sampling_frequency_index: SampleFrequencyIndex,
    pub channel_configuration: u8,
    pub frame_length: u16,
    // Format level, RTMP as such, informations.
    pub sound_format: u8,
    pub sound_rate: u8,
    pub sound_size: u8,
    pub sound_type: u8,
    // 0 for sh; 1 for raw data.
    pub aac_packet_type: u8,
    pub audio_specific_config: Bytes,
}

impl Default for RawAacStreamCodec {
    fn default() -> Self {
        Self {
            protection_absent: Default::default(),
            aac_object: Default::default(),
            sampling_frequency_index: Default::default(),
            channel_configuration: Default::default(),
            frame_length: Default::default(),
            sound_format: Default::default(),
            sound_rate: Default::default(),
            sound_size: Default::default(),
            sound_type: Default::default(),
            aac_packet_type: Default::default(),
            audio_specific_config: Bytes::default(),
        }
    }
}
