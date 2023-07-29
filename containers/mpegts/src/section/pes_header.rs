use std::{io::{Cursor, Seek, SeekFrom}, option};
use bytes::Bytes;
use anyhow::Result;
use byteorder::{ReadBytesExt, BigEndian};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StreamId {
    Audio(u8),
    Video(u8),
    Unknown(u8),
}

impl From<u8> for StreamId {
    fn from(v: u8) -> Self {
        match v {
            0xC0..=0xDF => StreamId::Audio(v & 0x1F),
            0xE0..=0xEF => StreamId::Video(v & 0x0F),
            _ => StreamId::Unknown(v),
        }
    }
}


#[derive(Clone, Debug)]
pub struct PesHeader {
    header_size: usize,
    stream_id: StreamId,
    size: usize,
    pts: Option<i64>,
    dts: Option<i64>
}

impl PesHeader {
    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PesHeader> {
        let mut header_size = 6;

        let start_code:u32 = reader.read_u24::<BigEndian>()?;
        assert_eq!(start_code, 1); //test for start code

        let stream_id = StreamId::from(reader.read_u8()?);
        let pes_length = reader.read_u16::<BigEndian>()?;

        reader.seek(SeekFrom::Current(1))?;

        let flags: u8 = reader.read_u8()?;
        let pes_ext_flag = flags & 0x01;
        let pes_crc_flag = (flags >> 1) & 0x01;
        let add_copy_info_flag = (flags >> 2) & 0x01;
        let dsm_trick_mode_flag = (flags >> 3) & 0x01;
        let es_rate_flag = (flags >> 4) & 0x01;
        let escr_flag = (flags >> 5) & 0x01;
        let pts_dts_flags = (flags >> 6) & 0x03;

        header_size += reader.read_u8()? as usize;

        Ok(PesHeader {
            header_size,
            stream_id,
            size: pes_length as usize,
            pts: None,
            dts: None,
        })
    }
}