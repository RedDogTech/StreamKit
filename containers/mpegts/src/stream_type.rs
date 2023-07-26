/// ETSI EN 300 468 V1.15.1 (2016-03)
/// ISO/IEC 13818-1

#[derive(Clone, Debug)]
pub enum StreamType {
    PrivateData, // PES packets containing private data
    AAC,
    H264,
    H265,
    AV1,
    OPUS,
    SCTE35,
    Unknown(u8),
}

impl From<u8> for StreamType {
    fn from(d: u8) -> Self {
        match d {
            0x06 => StreamType::PrivateData,
            0x0F => StreamType::AAC,
            0x1B => StreamType::H264,
            0x24 => StreamType::H265,
            0x86 => StreamType::SCTE35,
            0x9c => StreamType::OPUS,
            0x9f => StreamType::AV1,
            _ => StreamType::Unknown(d)
        }
    }
}

impl From<StreamType> for u8 {
    fn from(st: StreamType) -> u8 {
        match st {
            StreamType::PrivateData => 0x06,
            StreamType::AAC => 0x0F,
            StreamType::H264 => 0x1B,
            StreamType::H265 => 0x24,
            StreamType::SCTE35 => 0x86,
            StreamType::OPUS => 0x9c,
            StreamType::AV1 => 0x9f,
            StreamType::Unknown(d) => d,
        }
    }
}
