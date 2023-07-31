use mpegts::{demuxer::Demuxer, pid::Pid, stream_type::StreamType};
use time::{OffsetDateTime, Duration};

pub struct IngestDemuxer {
    latest_pcr_value: i64,
    latest_pcr_timestamp_90khz: u64,
    latest_pcr_datetime:  OffsetDateTime,
    first_pcr: bool,
}

impl Default for IngestDemuxer {
    fn default() -> Self {
        IngestDemuxer {
            first_pcr: false,
            latest_pcr_value: 0,
            latest_pcr_timestamp_90khz: 0,
            latest_pcr_datetime: (OffsetDateTime::now_utc()),
        }
    }
}

impl mpegts::DemuxerEvents for IngestDemuxer {
    fn on_program_streams(&mut self, pid: &Pid, stream_type: &StreamType) {
        log::info!("New stream found: {:?}, type:{:?}", pid, stream_type);
    }

    fn on_video_data(&mut self) {

    }

    fn on_audio_data(&mut self) {

    }

    fn on_pcr(&mut self, pcr: u64) {

        let prc_value: i64 = (pcr as i64 - mpegts::HZ as i64 + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
        
        // FIXME: this could be better :(
        if self.first_pcr {
            let pcr_diff = (prc_value - self.latest_pcr_value + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
            self.latest_pcr_timestamp_90khz += pcr_diff as u64;
            self.latest_pcr_datetime += Duration::seconds_f64(pcr_diff as f64 / mpegts::HZ as f64);

            println!("{}", self.latest_pcr_datetime);
        }
        
        self.first_pcr = true;
        self.latest_pcr_value = prc_value;   
    }
}

pub fn create_demux() -> Demuxer<IngestDemuxer> {
    Demuxer::new(IngestDemuxer::default())
}
