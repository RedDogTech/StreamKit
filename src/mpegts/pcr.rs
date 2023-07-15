use mpeg2ts_reader::{packet::{Packet, ClockRef}, demultiplex::PacketFilter};

use super::IngestDemuxContext;

pub struct PcrWatch(pub Option<ClockRef>);
impl PacketFilter for PcrWatch {
    type Ctx = IngestDemuxContext;
    fn consume(&mut self, _ctx: &mut Self::Ctx, pk: &Packet<'_>) {
        if let Some(af) = pk.adaptation_field() {
            if let Ok(pcr) = af.pcr() {
                Some(pcr);
            }
        }
    }
}
