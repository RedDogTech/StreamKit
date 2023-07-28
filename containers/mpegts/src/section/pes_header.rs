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

        let test:u32 = reader.read_u32::<BigEndian>()?;
        assert_eq!((test >> 8) & 0x00FFFFFF, 1); //test for start code

        let stream_id = StreamId::from(test as u8 & 0xFF);
        let pes_length = reader.read_u16::<BigEndian>()?;

        reader.seek(SeekFrom::Current(1))?;

        let pts_dts_flags = (reader.read_u8()? >> 6) & 0x03;
        header_size += reader.read_u8()? as usize;

        println!("pts_dts{}", pts_dts_flags);


        Ok(PesHeader {
            header_size,
            stream_id,
            size: pes_length as usize,
            pts: None,
            dts: None,
        })
    }
}