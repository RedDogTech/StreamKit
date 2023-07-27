use std::{collections::HashMap, io::{self, Seek}};
use crate::{pid::Pid, section::{pat::PAT, pmt::PMT, pes_header::PesHeader}, error::DemuxError, packet_header::{PacketHeader, AdaptationControl}, stream_type::StreamType};
use byteorder::{ReadBytesExt, BigEndian};
use anyhow::{Result, bail};

pub const SYNC_BYTE: u8 = 0x47;
pub const SIZE: usize = 188;

pub struct Demux {
    last_counter: u8,
    pmt_pid: Option<Pid>,
    pcr_pid: Option<Pid>,
    streams: HashMap<Pid, StreamType>,
}

impl Demux {

    pub fn new() -> Demux {
        Demux {
            last_counter: 0,
            pmt_pid: None,
            pcr_pid: None,
            streams: HashMap::new(),
        }
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

                        if self.streams.is_empty() {
                            self.streams = pmt.streams;
                        }
                    }
                }

                if let Some(stream_type) = self.streams.get(&header.pid) {
                    if header.adaptation_control.has_payload() {

                        if header.pusi {
                            let _ = PesHeader::try_new(&mut reader)?;
                        }
                        

                    }
                }
            }
        Ok(())
    }

    // pub fn demux_tables(&mut self, packet: Packet) {

    //     let pid = packet.pid;

    //     if pid.is_null() {
    //         log::warn!("ignoring packet");
    //         return;
    //     }

    //     match pid {
    //         Pid::PAT => {
    //             log::warn!("is PAT");
    //             let pat = PAT::try_new(&packet);

    //             println!("{:?}", pat);
    //         }
    //         Pid::Other(..) => {

    //         }
    //         _ => {}
    //     }
}
