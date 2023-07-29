use std::{collections::HashMap, io::{self, Seek}};
use crate::{pid::Pid, section::{pat::PAT, pmt::PMT, pes_header::PesHeader}, error::DemuxError, packet_header::{PacketHeader, AdaptationControl}, stream_type::StreamType, DemuxerEvents};
use byteorder::{ReadBytesExt, BigEndian};
use anyhow::{Result, bail};
use bytes::Bytes;

pub const SYNC_BYTE: u8 = 0x47;
pub const SIZE: usize = 188;

pub struct Demuxer<T> where T: DemuxerEvents {
    last_counter: u8,
    pmt_pid: Option<Pid>,
    pcr_pid: Option<Pid>,
    streams: HashMap<Pid, StreamType>,
    pmt: Option<PMT>,
    events: T,
}

impl<T> Demuxer<T> where T: DemuxerEvents {

    pub fn new(events: T) -> Demuxer<T> {
        Demuxer {
            last_counter: 0,
            pmt_pid: None,
            pcr_pid: None,
            streams: HashMap::new(),

            pmt: None,

            events,
        }
    }

    pub fn has_stream(&self, pid: &Pid) -> bool {
        if let Some(pmt) = &self.pmt {
            return pmt.streams.contains_key(pid);
        }
        false
    }

    pub fn push(&mut self, buf: &[u8]) ->Result<()> {
        let mut itr = buf
            .chunks_exact(SIZE);

            while let Some(packet) = itr.next() {
                let mut reader  = io::Cursor::new(bytes::Bytes::copy_from_slice(packet));
                let sync_byte = reader.read_u8()?;

                if sync_byte != SYNC_BYTE {
                    bail!(DemuxError::InvalidSyncByte{ expected: SYNC_BYTE, found: sync_byte});
                }

                let header = PacketHeader::try_new(&mut reader)?;

                // if header.counter != (self.last_counter + 1) {
                //     log::debug!("Incorrect continutiy expected: {} got {}", (self.last_counter + 1), header.counter);
                // }
                // if (self.last_counter + 1) == 15 {
                //     self.last_counter = 0;
                // } else {
                //     self.last_counter = header.counter;
                // }

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
                            for (key, value) in &pmt.streams { 
                                self.events.on_program_streams(&key, &value);
                            }

                            self.pmt = Some(pmt);
                        }
                    }
                }

                if self.has_stream(&header.pid) {
                    if header.adaptation_control.has_payload() {

                        if header.pusi {
                            let pes_header = PesHeader::try_new(&mut reader)?;
                            println!("pes_header={:?}", pes_header);
                        }
                        

                    }
                }
            }
        Ok(())
    }
}
