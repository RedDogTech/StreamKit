use std::io::{Cursor, Seek, SeekFrom};
use anyhow::Result;
use bytes::Bytes;
use byteorder::ReadBytesExt;

use crate::pid::Pid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AdaptationControl {
    Reserved,
    PayloadOnly,
    AdaptationFieldOnly,
    AdaptationFieldAndPayload,
}

impl AdaptationControl {
    fn from(val: u8) -> AdaptationControl {
        match val {
            0 => AdaptationControl::Reserved,
            1 => AdaptationControl::PayloadOnly,
            2 => AdaptationControl::AdaptationFieldOnly,
            3 => AdaptationControl::AdaptationFieldAndPayload,
            _ => panic!("invalid value {}", val),
        }
    }

    pub fn has_payload(&self) -> bool {
        match self {
            AdaptationControl::Reserved | AdaptationControl::AdaptationFieldOnly => false,
            AdaptationControl::PayloadOnly | AdaptationControl::AdaptationFieldAndPayload => true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PacketHeader {
    pub pid: Pid,
    pub pusi: bool,
    pub counter: u8,
    pub adaptation_control: AdaptationControl,
    pub pcr_flag: bool,
}

impl PacketHeader {

    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PacketHeader> {

        let second_byte = reader.read_u8()?;
        let third_byte = reader.read_u8()?;

        let pid: Pid = Pid::from(u16::from(second_byte & 0x1F) << 8 | u16::from(third_byte));
        let forth_byte = reader.read_u8()?;

        let adaptation_control = AdaptationControl::from((forth_byte & 0x30)>>4);
        let mut pcr_flag = false;

        if adaptation_control == AdaptationControl::AdaptationFieldOnly || adaptation_control == AdaptationControl::AdaptationFieldAndPayload
        {
            //TODO: build reader for this, at the moment we can
            //      greedly ginore all of it but move the reader
            //      pointer forward.
            let length = reader.read_u8()? as i64;

            if length != 0 {
                let adapt_fields = reader.read_u8()?;
                pcr_flag = (adapt_fields >> 4) & 0x01 != 0;

                reader.seek(SeekFrom::Current(length - 1))?;
            }
        }

        Ok(PacketHeader {
            pid,
            pusi: (second_byte & 0x40) != 0,
            counter: forth_byte & 0xf,
            adaptation_control,
            pcr_flag,
        })
    }
}