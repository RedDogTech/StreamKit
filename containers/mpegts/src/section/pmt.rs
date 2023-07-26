use std::io::{Cursor, Seek, SeekFrom};
use bytes::Bytes;
use anyhow::Result;
use byteorder::{ReadBytesExt, BigEndian};

use crate::{pid::Pid, stream_type::StreamType};

#[derive(Clone, Debug)]
pub struct Stream {
    stream_type: StreamType,
    elementry_pid: Pid,
}

#[derive(Clone, Debug)]
pub struct PMT {
    table_id: u8,
    program_number: u16,
    pcr_pid: Pid,
    streams: Vec<Stream>,
}

impl PMT {

    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PMT> {

        let table_id = reader.read_u8()?;
        let section_length = reader.read_u16::<BigEndian>()? & 0x0FFF;
        let program_number = reader.read_u16::<BigEndian>()?;

        let reserved = reader.read_u8()?;
        assert_eq!((reserved >> 6) & 0x03, 3);

        // seek past section_number and last_section_number
        reader.seek(SeekFrom::Current(2))?;

        let pcr_pid = reader.read_u16::<BigEndian>()? & 0x1FFF;
    
        let program_info_length = reader.read_u16::<BigEndian>()? &  0xFFF;

        if program_info_length > 0 {
            reader.seek(SeekFrom::Current(program_info_length as i64))?;
        }

        let mut remain_bytes = section_length - 4 - 9 - program_info_length;
        let mut streams = Vec::new();

        ////////
        //FIXME:    We should probbably use the readers own position
        //          rather than relying on different value for how
        //          we keep track of the remaning values. :(
        //////// 
        while remain_bytes > 0 {
            let stream_type = reader.read_u8()?;

            streams.push(Stream {
                stream_type: StreamType::from(stream_type),
                elementry_pid:  Pid::from(reader.read_u16::<BigEndian>()? & 0x1FFF),
            });
            
            //Skip over the extra info part
            let info_length = reader.read_u16::<BigEndian>()?  & 0x03FF;
            reader.seek(SeekFrom::Current(info_length as i64))?;

            remain_bytes -= 5 + info_length;
        }

        Ok(PMT {
            table_id,
            program_number,
            pcr_pid: Pid::from(pcr_pid),
            streams
        })
    }
}