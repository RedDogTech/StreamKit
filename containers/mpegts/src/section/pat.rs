use std::io::{Cursor, Seek, SeekFrom};
use bytes::Bytes;
use anyhow::Result;
use byteorder::{ReadBytesExt, BigEndian};

use crate::pid::Pid;

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Program {
    program_number: u16,
    pub program_pid: Pid,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct PAT {
    table_id: u8,
    section_length: u16,
    stream_id: u16,
    pub programs: Vec<Program>
}

impl PAT {

    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PAT> {

        let table_id = reader.read_u8()?;
        let section_length = reader.read_u16::<BigEndian>()? & 0x0FFF;
        let stream_id = reader.read_u16::<BigEndian>()?;

        let reserved = reader.read_u8()?;
        assert_eq!((reserved >> 6) & 0x03, 3);

        // skip section_number and last_section_number
        reader.seek(SeekFrom::Current(2))?;

        let program_count = section_length - 5 /*Header */ -4 /* crc */;
        let mut programs = vec![];

        for _ in 0..(program_count / 4) {
            let program_number = reader.read_u16::<BigEndian>()?;
            let program_pid = Pid::from(reader.read_u16::<BigEndian>()? & 0x1fff);

            programs.push(Program {
                program_number,
                program_pid
            });
        }

        Ok(PAT {
            table_id,
            section_length,
            stream_id,
            programs
        })
    }
}