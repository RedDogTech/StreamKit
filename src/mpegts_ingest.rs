use std::sync::Arc;

use aac::AacCoder;
use bytes::{Bytes, BytesMut};
use common::FormatReader;
use h264::H264Coder;
use mpegts::{demuxer::Demuxer, pid::Pid, stream_type::StreamType};
use time::{OffsetDateTime, Duration};

use crate::segment_store::SegmentStore;

pub struct IngestDemuxer {
    latest_pcr_value: i64,
    latest_pcr_timestamp_90khz: u64,
    latest_pcr_datetime:  OffsetDateTime,
    first_pcr: bool,

    segment_store: SegmentStore,

    h264_coder: H264Coder,
    aac_coder: AacCoder,
}

impl IngestDemuxer {
    fn new() -> Self {
        IngestDemuxer {
            first_pcr: false,
            latest_pcr_value: 0,
            latest_pcr_timestamp_90khz: 0,
            latest_pcr_datetime: (OffsetDateTime::now_utc()),

            segment_store: SegmentStore::default(),

            h264_coder: H264Coder::new(),
            aac_coder: AacCoder::new(),
        }
    }
}

impl mpegts::DemuxerEvents for IngestDemuxer {
    fn on_program_streams(&mut self, pid: &Pid, stream_type: &StreamType) {
        log::info!("New stream found: {:?}, type:{:?}", pid, stream_type);
    }

    fn on_video_data(&mut self, data: Bytes, pts: Option<u64>, dts: Option<u64>) {   

        if self.first_pcr {
            let pts: i64 = pts.unwrap() as i64; //Should not fail
            let dts: i64 = match dts {
                Some(dts) => dts as i64,
                None => pts as i64,
            };

            let cts: i64 = (pts - dts + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
            let timestamp: i64 = ((dts as i64 - self.latest_pcr_value as i64 + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64) + self.latest_pcr_timestamp_90khz as i64;
            let program_date_time = self.latest_pcr_datetime + Duration::seconds_f64(((dts as f64 - self.latest_pcr_value as f64 + mpegts::PCR_CYCLE as f64) % mpegts::PCR_CYCLE as f64) / mpegts::HZ as f64);
            
            println!("cts:{}, timestamp:{} program_date_time:{}", cts, timestamp, program_date_time);
        }

        let video = match self.h264_coder.read_format(h264::AnnexB, &data).unwrap() {
            Some(avc) => println!("{:?}", avc),
            None => {},
        };

        if let Some(idr) = &self.h264_coder.dcr {
            let _ = self.segment_store.init_video_stsd(idr.clone());

        }
    }

    fn on_audio_data(&mut self, data: Bytes, pts: Option<u64>) {
        // let audio = match self.aac_coder.read_format(aac::AudioDataTransportStream, &data).unwrap() {
        //     Some(aac) => println!("{:?}", aac),
        //     None => {},
        // };
    }

    fn on_pcr(&mut self, pcr: u64) {
        let prc_value: i64 = (pcr as i64 - mpegts::HZ as i64 + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
        
        // FIXME: this could be better :(
        if self.first_pcr {
            let pcr_diff = (prc_value - self.latest_pcr_value + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
            self.latest_pcr_timestamp_90khz += pcr_diff as u64;
            self.latest_pcr_datetime += Duration::seconds_f64(pcr_diff as f64 / mpegts::HZ as f64);
        }
        
        self.first_pcr = true;
        self.latest_pcr_value = prc_value;   
    }
}

pub async fn create_demux() -> Demuxer<IngestDemuxer> {
    Demuxer::new(IngestDemuxer::new())
}
