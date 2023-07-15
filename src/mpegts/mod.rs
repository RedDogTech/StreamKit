use std::collections::HashMap;

use mpeg2ts_reader::{
    demultiplex::{FilterRequest, NullPacketFilter, PatPacketFilter, PmtPacketFilter, FilterChangeset, Demultiplex, DemuxContext},
    packet::{Pid, ClockRef}
};

use crate::store;

use self::scte::{PcrWatch, Scte35StreamConsumer};

pub mod scte;

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
        //Adts: pes::PesPacketFilter<IngestDemuxContext, adts::AdtsElementaryStreamConsumer>,
    }
}

pub fn create_demux(store: store::Store) -> (IngestDemuxContext, Demultiplex<IngestDemuxContext>) {
    let mut ctx = IngestDemuxContext::new(store);
    let demux = Demultiplex::new(&mut ctx);
    (ctx, demux)
}

pub struct IngestDemuxContext {
    changeset: FilterChangeset<IngestFilterSwitch>,
    store: store::Store,
    last_pcrs: HashMap<Pid, Option<ClockRef>>,
}

impl IngestDemuxContext {
    pub fn new(store: store::Store) -> IngestDemuxContext {
        IngestDemuxContext {
            store,
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

            // demultiplex::FilterRequest::ByStream {
            //     program_pid, stream_type: StreamType::Adts, pmt, stream_info,
            // } => IngestFilterSwitch::Adts(adts::AdtsElementaryStreamConsumer::construct(stream_info, self.store.clone())),
            
            FilterRequest::ByStream { program_pid, stream_type: scte35_reader::SCTE35_STREAM_TYPE, pmt,stream_info} => {
                log::warn!("by stream!");
                Scte35StreamConsumer::construct(self.last_pcr(program_pid), program_pid, pmt, stream_info)
            },

            FilterRequest::Pmt {pid, program_number} => {
                // prepare structure needed to print PCR values later on
                self.last_pcrs.insert(pid, None);
                IngestFilterSwitch::Pmt(self.construct_pmt(pid, program_number))
            }

            FilterRequest::ByStream { program_pid, .. } => {
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
