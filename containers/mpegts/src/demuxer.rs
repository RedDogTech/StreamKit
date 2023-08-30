use std::{collections::HashMap, io::{self, Seek}};
use crate::{pid::Pid, section::{pat::PAT, pmt::PMT, pes_header::{PesHeader, StreamId}}, error::DemuxError, packet_header::PacketHeader, DemuxerEvent};
use byteorder::ReadBytesExt;
use anyhow::{Result, bail};
use bytes::{BytesMut, BufMut, Bytes};
use bytesio::bytes_reader::BytesCursor;

pub const SYNC_BYTE: u8 = 0x47;
pub const SIZE: usize = 188;

#[derive(Clone, Debug)]
struct Packet {
    buffer: BytesMut,
    stream_id: StreamId,
    pts: Option<u64>,
    dts: Option<u64>,
}

pub struct Demuxer {
    pmt_pid: Option<Pid>,
    pcr_pid: Option<Pid>,
    pmt: Option<PMT>,

    return_queue: Vec<DemuxerEvent>,
    continutiy_check: HashMap<Pid, u8>,
    packets: HashMap<Pid, Packet>,
}


pub static ANNEXB_NALUSTART_CODE: Bytes = Bytes::from_static(&[0x00, 0x00, 0x00, 0x01]);

impl Demuxer {

    pub fn new() -> Demuxer {
        Demuxer {   
            pmt_pid: None,
            pcr_pid: None,

            pmt: None,

            continutiy_check: HashMap::new(),
            packets: HashMap::new(),
            return_queue: Vec::new(),
        }
    }

    pub fn has_stream(&self, pid: &Pid) -> bool {
        if let Some(pmt) = &self.pmt {
            return pmt.streams.contains_key(pid);
        }
        false
    }

    fn emit(&mut self, event: DemuxerEvent) {
        self.return_queue.push(event);
    }

    pub fn check_continutiy(&mut self, packet_header: &PacketHeader) {
        if self.continutiy_check.contains_key(&packet_header.pid) {

            let last_counter = self.continutiy_check.get_mut(&packet_header.pid).unwrap();

            let mut expected_count = 0;

            if *last_counter < 0x0f {
                expected_count = *last_counter + 1;
            }

            if packet_header.continuity_counter != expected_count {
                log::debug!("An out-of-order packet was received.(PID : {:?} Expected : {}, Received : {}", packet_header.pid, expected_count, packet_header.continuity_counter);
            }

            *last_counter = expected_count;
        } else {
            self.continutiy_check.insert(packet_header.pid, packet_header.continuity_counter);
        }
    }

    pub fn push(&mut self, buf: &[u8]) ->Result<Vec<DemuxerEvent>> {
        let mut itr = buf
            .chunks_exact(SIZE);

            while let Some(packet) = itr.next() {
                let mut reader  = io::Cursor::new(bytes::Bytes::copy_from_slice(packet));
                let sync_byte = reader.read_u8()?;

                if sync_byte != SYNC_BYTE {
                    bail!(DemuxError::InvalidSyncByte{ expected: SYNC_BYTE, found: sync_byte});
                }

                let header = PacketHeader::try_new(&mut reader)?;

                if let Some(pcr) = header.pcr {
                    self.emit(DemuxerEvent::ClockRef(pcr));
                }

                if header.pid == Pid::PAT {
                    if header.adaptation_control.has_payload() {
                        // skip pointer field
                        reader.seek(io::SeekFrom::Current(1))?;
                    }

                    let pat = PAT::try_new(&mut reader)?;

                    //We assume a single program
                    if self.pmt_pid == None {
                        let pid: Pid = pat.programs[0].program_pid;
                        log::debug!("Found PMT pid:{:?}", pid);
                        self.pmt_pid = Some(pid);
                    }
                }

                if let Some(pid) = self.pmt_pid {
                    if pid == header.pid {
                        if header.adaptation_control.has_payload() {
                            // skip pointer field
                            reader.seek(io::SeekFrom::Current(1))?;
                        }
                        
                        let pmt = PMT::try_new(&mut reader)?;
                        self.pcr_pid = Some(pmt.pcr_pid);

                        if self.pmt.is_none() {
                            self.emit(DemuxerEvent::StreamDetails(pmt.streams.clone()));
                            self.pmt = Some(pmt);
                        }
                    }
                }

                if self.has_stream(&header.pid) {
                    self.check_continutiy(&header);

                    if header.adaptation_control.has_payload() {

                        if header.pusi {
                            if let Some(packet) = self.packets.get_mut(&header.pid) {
                                if packet.buffer.len() != 0 {
                                    let packet_clone = packet.clone();

                                    let stream_type = self.pmt.clone().unwrap().streams[&header.pid].clone();

                                    match packet.stream_id {
                                        StreamId::Audio(_) => {
                                            self.emit(DemuxerEvent::Audio(stream_type, packet_clone.buffer.freeze(), packet_clone.pts));
                                        }
                                        StreamId::Video(_) => {
                                            self.emit(DemuxerEvent::Video(stream_type, packet_clone.buffer.freeze(), packet_clone.pts, packet_clone.dts));
                                        },
                                        _ => (),
                                    }
                                }
                            }

                            let pes_header = PesHeader::try_new(&mut reader)?;
                            let packet = Packet {
                                buffer: BytesMut::new(),
                                stream_id: pes_header.stream_id,
                                pts: pes_header.pts,
                                dts: pes_header.dts,
                            };

                            self.packets.insert(header.pid, packet);
                        }

                        if let Some(packet) = self.packets.get_mut(&header.pid) {
                            let remaning = reader.get_remaining();
                            packet.buffer.put(remaning);
                        }
                    }
                }
            }

            Ok(self.return_queue.drain(..).collect())
    }
}
