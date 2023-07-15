use mpeg2ts_reader::{
    demultiplex::{ NullPacketFilter, PacketFilter},
    packet::{Pid, ClockRef, Packet}, psi::{self, BufferCompactSyntaxParser, pmt::{PmtSection, StreamInfo}}
};

use scte35_reader::{self, SpliceInfoProcessor, SpliceInfoHeader, SpliceCommand, SpliceDescriptors, SpliceInsert, SpliceMode, SpliceTime, Scte35SectionProcessor};

use super::{IngestFilterSwitch, IngestDemuxContext};

pub struct DumpSpliceInfoProcessor {
    pub elementary_pid: Option<Pid>,
    pub last_pcr: Option<ClockRef>,
}

impl SpliceInfoProcessor for DumpSpliceInfoProcessor {
    fn process(
        &self,
        header: SpliceInfoHeader<'_>,
        command: SpliceCommand,
        descriptors: SpliceDescriptors<'_>,
    ) {
        if let Some(elementary_pid) = self.elementary_pid {
            log::warn!("{:?} ", elementary_pid);
        }

        if let Some(pcr) = self.last_pcr {
            log::warn!("Last {:?}: ", pcr)
        }

        print!("{:?} {:#?}", header, command);
        if let SpliceCommand::SpliceInsert { splice_detail, .. } = command {
            if let SpliceInsert::Insert { splice_mode, .. } = splice_detail {
                if let SpliceMode::Program(SpliceTime::Timed(t)) =
                    splice_mode
                {
                    if let Some(time) = t {
                        let time_ref = ClockRef::from_parts(time, 0);
                        if let Some(pcr) = self.last_pcr {
                            let mut diff = time_ref.base() as i64 - pcr.base() as i64;
                            if diff < 0 {
                                diff += (std::u64::MAX / 2) as i64;
                            }
                            log::warn!(" {}ms after most recent PCR", diff / 90);
                        }
                    }
                }
            }
        }

        for d in &descriptors {
            log::warn!(" - {:#?}", d);
        }
    }
}

pub struct Scte35StreamConsumer {
    section: psi::SectionPacketConsumer<
        psi::CompactSyntaxSectionProcessor<
            psi::BufferCompactSyntaxParser<
                Scte35SectionProcessor<DumpSpliceInfoProcessor, IngestDemuxContext>,
            >,
        >,
    >,
}

impl Scte35StreamConsumer {
    fn new(elementary_pid: Pid, last_pcr: Option<ClockRef>) -> Self {
        let parser =
            Scte35SectionProcessor::new(DumpSpliceInfoProcessor {
                elementary_pid: Some(elementary_pid),
                last_pcr
            });
        Scte35StreamConsumer {
            section: psi::SectionPacketConsumer::new(psi::CompactSyntaxSectionProcessor::new(
                BufferCompactSyntaxParser::new(parser),
            )),
        }
    }

    pub fn construct(
        last_pcr: Option<ClockRef>,
        program_pid: Pid,
        pmt: &PmtSection<'_>,
        stream_info: &StreamInfo<'_>,
    ) -> IngestFilterSwitch {
        if scte35_reader::is_scte35(pmt) {
            println!(
                "Program {:?}: Found SCTE-35 data on {:?} ({:#x})",
                program_pid,
                stream_info.elementary_pid(),
                u16::from(stream_info.elementary_pid())
            );
            IngestFilterSwitch::Scte35(Scte35StreamConsumer::new(stream_info.elementary_pid(), last_pcr))
        } else {
            println!("Program {:?}: {:?} has type {:?}, but PMT lacks 'CUEI' registration_descriptor that would indicate SCTE-35 content",
                     program_pid,
                     stream_info.elementary_pid(),
                     stream_info.stream_type());
                     IngestFilterSwitch::Null(NullPacketFilter::default())
        }
    }
}
impl PacketFilter for Scte35StreamConsumer {
    type Ctx = IngestDemuxContext;
    fn consume(&mut self, ctx: &mut Self::Ctx, pk: &Packet<'_>) {
        self.section.consume(ctx, pk);
    }
}

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
