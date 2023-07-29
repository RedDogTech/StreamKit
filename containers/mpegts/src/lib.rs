pub mod error;
pub mod stream_type;
pub mod pid;
pub mod packet_header;
pub mod pes;
pub mod demuxer;
pub mod section;

pub trait DemuxerEvents {
    fn on_program_streams(&mut self) {}
    fn on_video_data(&mut self) {}
    fn on_audio_data(&mut self) {}
    fn on_pcr(&mut self) {}
}