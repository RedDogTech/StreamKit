use std::collections::HashMap;
use bytes::Bytes;

const SYNC_BYTE: u8 = 0x47;
const PACKET_SIZE: u8 = 187;

pub struct Packet {
    pub pid: u32,

    pub offset: usize,

    /// presentation time stamp
    pub pts: Option<u32>,

    /// decode time stamp
    pub dts: Option<u32>,

    pub buf: Bytes,

    /// got ts PUSI
    started: bool,
}

impl Packet {
    fn new(pid: u32) -> Packet {
        Packet {
            pid,
            offset: 0,
            pts: None,
            dts: None,
            buf: Default::default(),
            started: false,
        }
    }
}

#[derive(Default)]
struct Packets(HashMap<u32, Packet>);