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

pub const HZ: u32 = 90000;
pub const PCR_CYCLE: u64 = 8589934592; // 2 ** 33

#[allow(unused_variables)]
// pub trait DemuxerEvents {
//     fn on_program_streams(&mut self, pid: &Pid, stream_type: &StreamType) {}
//     fn on_video_data(&mut self, data: Bytes, pts: Option<u64>, dts: Option<u64>) {}
//     fn on_audio_data(&mut self, data: Bytes, pts: Option<u64>) {}
//     fn on_pcr(&mut self, clock: u64) {}
// }



#[derive(Clone, Debug)]
pub enum DemuxerEvent {
    StreamDetails(HashMap<Pid, StreamType>),
    Video(Bytes, Option<u64>, Option<u64>),
    Audio(Bytes, Option<u64>),
    ClockRef(u64),
}
