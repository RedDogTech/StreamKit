use std::collections::HashMap;

use bytes::Bytes;
use pid::Pid;
use stream_type::StreamType;

pub mod error;
pub mod stream_type;
pub mod pid;
pub mod packet_header;
pub mod demuxer;
pub mod section;

pub const HZ: u32 = 90_000;
pub const PCR_CYCLE: u64 = 8_589_934_592; // 2 ** 33


#[derive(Clone, Debug)]
pub enum DemuxerEvent {
    StreamDetails(HashMap<Pid, StreamType>),
    Video(Bytes, Option<u64>, Option<u64>),
    Audio(Bytes, Option<u64>),
    ClockRef(u64),
}
