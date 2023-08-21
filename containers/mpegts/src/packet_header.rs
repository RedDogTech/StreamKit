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
    pub continuity_counter: u8,
    pub adaptation_control: AdaptationControl,
    pub pcr: Option<u64>,
    pub header_size: i64,
}

impl PacketHeader {

    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PacketHeader> {
        let mut header_size = 4;

        let second_byte = reader.read_u8()?;
        let third_byte = reader.read_u8()?;

        let pid: Pid = Pid::from(u16::from(second_byte & 0x1F) << 8 | u16::from(third_byte));
        let forth_byte = reader.read_u8()?;

        let adaptation_control = AdaptationControl::from((forth_byte & 0x30)>>4);
        let mut pcr = None;

        if adaptation_control == AdaptationControl::AdaptationFieldOnly || adaptation_control == AdaptationControl::AdaptationFieldAndPayload
        {
            //TODO: build reader for this, at the moment we can
            //      greedly ginore all of it but move the reader
            //      pointer forward.
            let mut adapt_length = reader.read_u8()? as i64;

            if adapt_length != 0 {
                let adapt_fields = reader.read_u8()?;
                adapt_length -= 1;

                let pcr_flag = (adapt_fields >> 4) & 0x01 != 0;

                if pcr_flag {
                    pcr = Some(Self::read_pcr(reader)?);
                    adapt_length -= 7;
                } 

                reader.seek(SeekFrom::Current(adapt_length))?;
                header_size += adapt_length;
            }
        }

        Ok(PacketHeader {
            pid,
            pusi: (second_byte & 0x40) != 0,
            continuity_counter: forth_byte & 0xf,
            adaptation_control,
            pcr,
            header_size,
        })
    }

    #[inline(always)]
    fn read_pcr(reader: &mut Cursor<Bytes>) -> Result<u64> {
        let mut pcr :u64 = 0;
        let mut val :u64 = reader.read_u8()? as u64;

        pcr |= (val << 25) & 0x1FE000000;
    
        val = reader.read_u8()? as u64;
        pcr |= (val << 17) & 0x1FE0000;
    
        val = reader.read_u8()? as u64;
        pcr |= (val << 9) & 0x1FE00;
    
        val = reader.read_u8()? as u64;
        pcr |= (val << 1) & 0x1FE;
    
        val = reader.read_u8()? as u64;
        pcr |= (val >> 7) & 0x01;
    
        let _ext = (reader.read_u8()? as u16 & 0b1) << 8 | reader.read_u8()? as u16;
    
        Ok(pcr)
    }

}