use bytes::Bytes;
use pid::Pid;
use stream_type::StreamType;

pub mod error;
pub mod stream_type;
pub mod pid;
pub mod packet_header;
pub mod demuxer;
pub mod section;

pub const HZ: u64 = 90000;
pub const PCR_CYCLE: u64 = 8589934592; // 2 ** 33

pub trait DemuxerEvents {
    fn on_program_streams(&mut self, pid: &Pid, stream_type: &StreamType) {}
    fn on_video_data(&mut self) {}
    fn on_audio_data(&mut self) {}
    fn on_pcr(&mut self, clock: u64) {}
}

pub struct Packet {
    data: Bytes,
    pts: Option<u64>,
    dts: Option<u64>,
    
    stream_type: StreamType,
}
