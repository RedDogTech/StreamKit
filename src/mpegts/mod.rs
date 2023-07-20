pub mod scte;
pub mod pcr;
pub mod adts;

pub mod codec;

use std::collections::HashMap;

use mpeg2ts_reader::{
    demultiplex::{FilterRequest, NullPacketFilter, PatPacketFilter, PmtPacketFilter, FilterChangeset, Demultiplex, DemuxContext, self},
    packet::{Pid, ClockRef}, pes::{Timestamp, self}, StreamType
};

use crate::store;
use self::{scte::Scte35StreamConsumer, pcr::PcrWatch};


mpeg2ts_reader::packet_filter_switch! {
    IngestFilterSwitch<IngestDemuxContext> {
        // Handle MPEG Stream
        Pat: PatPacketFilter<IngestDemuxContext>,
        Pmt: PmtPacketFilter<IngestDemuxContext>,
        Null: NullPacketFilter<IngestDemuxContext>,

        // Handle Ads
        Scte35: Scte35StreamConsumer,
        Pcr: PcrWatch,

        //Handle the codecs
        //H264: pes::PesPacketFilter<IngestDemuxContext, h264::H264ElementaryStreamConsumer>,
        //H265: pes::PesPacketFilter<IngestDemuxContext, h265::H265ElementaryStreamConsumer>,
        Adts: pes::PesPacketFilter<IngestDemuxContext, adts::AdtsElementaryStreamConsumer>,
    }
}

pub fn create_demux() -> (IngestDemuxContext, Demultiplex<IngestDemuxContext>) {
    let mut ctx = IngestDemuxContext::new();
    let demux = Demultiplex::new(&mut ctx);
    (ctx, demux)
}

pub struct IngestDemuxContext {
    changeset: FilterChangeset<IngestFilterSwitch>,
    //store: store::Store,
    last_pcrs: HashMap<Pid, Option<ClockRef>>,
}

impl IngestDemuxContext {
    pub fn new() -> IngestDemuxContext {
        IngestDemuxContext {
            //store,
            changeset: Default::default(),
            last_pcrs: HashMap::new(),
        }
    }

    fn construct_pmt(&self, pid: Pid, program_number: u16) -> PmtPacketFilter<IngestDemuxContext> {
        log::debug!("new pid {:?}", pid);

        PmtPacketFilter::new(
            pid,
            program_number,
        )
    }

    pub fn last_pcr(&self, program_pid: Pid) -> Option<ClockRef> {
        self.last_pcrs
            .get(&program_pid)
            .expect("last_pcrs entry didn't exist on call to last_pcr()")
            .clone()
    }

}
impl DemuxContext for IngestDemuxContext {
    type F = IngestFilterSwitch;

    fn filter_changeset(&mut self) -> &mut FilterChangeset<Self::F> {
        &mut self.changeset
    }
    fn construct(&mut self, req: FilterRequest<'_, '_>) -> Self::F {
        match req {
            FilterRequest::ByPid(Pid::PAT) => {
                IngestFilterSwitch::Pat(PatPacketFilter::default())
            }

            // demultiplex::FilterRequest::ByStream {
            //     program_pid, stream_type: StreamType::H264, pmt, stream_info,
            // } => IngestFilterSwitch::H264(h264::H264ElementaryStreamConsumer::construct(stream_info, self.store.clone())),

            demultiplex::FilterRequest::ByStream {
                program_pid, stream_type: StreamType::Adts, pmt, stream_info,
            } => IngestFilterSwitch::Adts(adts::AdtsElementaryStreamConsumer::construct(stream_info)),
            
            FilterRequest::ByStream { program_pid, stream_type: scte35_reader::SCTE35_STREAM_TYPE, pmt,stream_info} => {
                log::warn!("by stream!");
                Scte35StreamConsumer::construct(self.last_pcr(program_pid), program_pid, pmt, stream_info)
            },

            FilterRequest::Pmt {pid, program_number} => {
                self.last_pcrs.insert(pid, None);
                IngestFilterSwitch::Pmt(self.construct_pmt(pid, program_number))
            }

            FilterRequest::ByStream { program_pid, stream_info, .. } => {
                log::warn!("{:?}",stream_info);
                IngestFilterSwitch::Pcr(PcrWatch(self.last_pcr(program_pid)))
            }

            FilterRequest::ByPid(_) => {
                IngestFilterSwitch::Null(NullPacketFilter::default())
            }

            FilterRequest::Nit { .. } => {
                IngestFilterSwitch::Null(NullPacketFilter::default())
            }
        }
    }
}

struct UnwrapTimestamp {
    last: Option<Timestamp>,
    carry: u64,
}
impl Default for UnwrapTimestamp {
    fn default() -> Self {
        UnwrapTimestamp {
            last: None,
            carry: 0
        }
    }
}
impl UnwrapTimestamp {
    /// Panics if the `update()` method as never been called
    fn unwrap(&self, ts: Timestamp) -> i64 {
        // check invariant,
        assert_eq!(self.carry & Timestamp::MAX.value(), 0);

        let last = self.last.expect("No previous call to update");
        let diff = ts.value() as i64 - last.value() as i64;
        let half = (Timestamp::MAX.value() / 2) as i64;
        if diff > half {
            ts.value() as i64 + self.carry as i64 - (Timestamp::MAX.value() + 1) as i64
        } else if diff < -(half as i64) {
            ts.value() as i64 + self.carry as i64 + (Timestamp::MAX.value() + 1) as i64
        } else {
            ts.value() as i64 + self.carry as i64
        }
    }

    fn update(&mut self, ts: Timestamp) {
        if let Some (last) = self.last {
            let half = (Timestamp::MAX.value() / 2) as i64;
            let diff = ts.value() as i64 - last.value() as i64;
            if diff < -half {
                self.carry += Timestamp::MAX.value() + 1;
            }
        }
        self.last = Some(ts);
    }
}