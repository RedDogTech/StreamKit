use bytes::Bytes;
use pid::Pid;
use stream_type::StreamType;

pub mod error;
pub mod stream_type;
pub mod pid;
pub mod packet_header;
pub mod demuxer;
pub mod section;

pub trait DemuxerEvents {
    fn on_program_streams(&mut self, pid: &Pid, stream_type: &StreamType) {}
    fn on_video_data(&mut self) {}
    fn on_audio_data(&mut self) {}
    fn on_pcr(&mut self) {}
}

pub struct Packet {
    data: Bytes,
    pts: Option<u64>,
    dts: Option<u64>,
    
    stream_type: StreamType,
}
