use std::io::{Cursor, Seek, SeekFrom};
use bytes::Bytes;
use anyhow::Result;
use byteorder::{ReadBytesExt, BigEndian};

#[derive(Clone, Debug)]
pub struct PesHeader {

}

impl PesHeader {
    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PesHeader> {

        let test:u32 = reader.read_u32::<BigEndian>()?;
        assert_eq!((test >> 8) & 0x00FFFFFF, 1); //test for start code
        println!("packet_start_code ={}", (test >> 8) & 0x00FFFFFF);
        println!("stream_id ={}", test & 0xFF);

        let pes_length = reader.read_u16::<BigEndian>()?;
        println!("pes_length ={}", pes_length);
        Ok(PesHeader {

        })
    }
}