use std::io::{Cursor, Seek, SeekFrom};
use anyhow::Result;
use bytes::Bytes;
use byteorder::ReadBytesExt;

use crate::pid::Pid;

#[derive(Clone, Debug)]
pub struct PacketHeader {
    pub pid: Pid,
    pub pusi: bool,
    pub payload: bool,
    pub counter: u8,
}

impl PacketHeader {

    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PacketHeader> {

        let second_byte = reader.read_u8()?;
                let third_byte = reader.read_u8()?;
                let pid: Pid = Pid::from(u16::from(second_byte & 0x1F) << 8 | u16::from(third_byte));
                let forth_byte = reader.read_u8()?;

                //AFC = 01 -> only payload
                //AFC = 10 -> only adaptation field
                //AFC = 11 -> payload & adaptation field
                let afc = (((forth_byte & 0x30)>>4) == 0x2) || (((forth_byte & 0x30)>>4) == 0x3);

                if afc
                {
                    //TODO: build reader for this, at the moment we can
                    //      greedly ginore all of it but move the reader
                    //      pointer forward.
                    let length = reader.read_u8()? as i64;
                    reader.seek(SeekFrom::Current(length))?;
                }

        Ok(PacketHeader {
            pid,
            pusi: (second_byte & 0x40) != 0,
            payload: (((forth_byte&0x30)>>4) == 0x1) || (((forth_byte&0x30)>>4) == 0x3),
            counter: forth_byte & 0xf
        })
    }
}