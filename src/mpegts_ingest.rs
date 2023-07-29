use mpegts::{demuxer::Demuxer, pid::Pid, stream_type::StreamType};




impl Default for IngestDemuxer {
    fn default() -> Self {
        IngestDemuxer {
        }
    }
}

pub struct IngestDemuxer {
}

impl mpegts::DemuxerEvents for IngestDemuxer {
    fn on_program_streams(&mut self, pid: &Pid, stream_type: &StreamType) {
        log::info!("New stream found: {:?}, type:{:?}", pid, stream_type);
    }

    fn on_video_data(&mut self) {

    }

    fn on_audio_data(&mut self) {

    }

    fn on_pcr(&mut self) {

    }
}

pub fn create_demux() -> Demuxer<IngestDemuxer> {
    Demuxer::new(IngestDemuxer::default())
}
