use {
    super::AacError,
    bytes::Buf,
    std::{convert::TryFrom, io::Cursor},
};

// Bits | Description
// ---- | -----------
// 5    | Audio object type
// 4    | Sampling frequency index
// 4    | Channel configuration
// AOT specific section
// 1    | Frame length flag
// 1    | Depends on core coder
// 1    | Extension flag
///
#[derive(Debug, Clone, PartialEq, Copy, Eq)]
pub struct AudioSpecificConfiguration {
    pub object_type: AudioObjectType,
    pub sampling_frequency_index: SamplingFrequencyIndex,
    pub sampling_frequency: Option<u32>,
    pub channel_configuration: ChannelConfiguration,
    pub frame_length_flag: bool,
    pub depends_on_core_coder: bool,
    pub extension_flag: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingFrequencyIndex(u8);

impl From<SamplingFrequencyIndex> for u8 {
    fn from(val: SamplingFrequencyIndex) -> Self {
        val.0
    }
}

impl TryFrom<u8> for SamplingFrequencyIndex {
    type Error = AacError;

    fn try_from(val: u8) -> Result<Self, AacError> {
        match val {
            0..=12 | 15 => Ok(Self(val)),
            _ => Err(AacError::UnsupportedFrequencyIndex(val)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelConfiguration(u8);

impl From<ChannelConfiguration> for u8 {
    fn from(val: ChannelConfiguration) -> Self {
        val.0
    }
}

impl TryFrom<u8> for ChannelConfiguration {
    type Error = AacError;

    fn try_from(val: u8) -> Result<Self, AacError> {
        match val {
            0..=7 => Ok(Self(val)),
            _ => Err(AacError::UnsupportedChannelConfiguration(val)),
        }
    }
}

// See [MPEG-4 Audio Object Types][audio_object_types]
//
// [audio_object_types]: https://en.wikipedia.org/wiki/MPEG-4_Part_3#MPEG-4_Audio_Object_Types
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioObjectType {
    Reserved = 0,
    AacMain = 1,
    AacLowComplexity = 2,
    AacScalableSampleRate = 3,
    AacLongTermPrediction = 4,
}

impl Default for AudioObjectType {
    fn default() -> Self {
        Self::Reserved
    }
}

impl TryFrom<u16> for AudioObjectType {
    type Error = AacError;

    fn try_from(value: u16) -> Result<Self, AacError> {
        Ok(match value {
            1 => Self::AacMain,
            2 => Self::AacLowComplexity,
            3 => Self::AacScalableSampleRate,
            4 => Self::AacLongTermPrediction,
            0 => Self::Reserved,
            _ => return Err(AacError::UnsupportedAudioFormat),
        })
    }
}

impl Into<u8> for AudioObjectType {
    fn into(self) -> u8 {
        match self {
            Self::AacMain => 1,
            Self::AacLowComplexity => 2,
            Self::AacScalableSampleRate => 3,
            Self::AacLongTermPrediction => 4,
            Self::Reserved => 0,
        }
    }
}

impl TryFrom<&[u8]> for AudioSpecificConfiguration {
    type Error = AacError;

    fn try_from(val: &[u8]) -> Result<Self, Self::Error> {
        if val.len() < 2 {
            return Err(AacError::NotEnoughData("AAC audio specific config"));
        }

        let mut buf = Cursor::new(val);

        let header_a = buf.get_u8();
        let header_b = buf.get_u8();

        let object_type = AudioObjectType::try_from((header_a & 0xF8) as u16 >> 3)?;

        let sf_idx = ((header_a & 0x07) << 1) | (header_b >> 7);
        let sampling_frequency_index = SamplingFrequencyIndex::try_from(sf_idx)?;

        let channel_configuration = ChannelConfiguration::try_from((header_b >> 3) & 0x0F)?;
        let frame_length_flag = (header_b & 0x04) == 0x04;
        let depends_on_core_coder = (header_b & 0x02) == 0x02;
        let extension_flag = (header_b & 0x01) == 0x01;

        Ok(Self {
            object_type,
            sampling_frequency_index,
            sampling_frequency: None,
            channel_configuration,
            frame_length_flag,
            depends_on_core_coder,
            extension_flag,
        })
    }
}
