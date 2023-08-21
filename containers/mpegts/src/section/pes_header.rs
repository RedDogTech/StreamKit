use std::io::{Cursor, Seek, SeekFrom};
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


#[allow(unused)]
#[derive(Clone, Debug)]
pub struct PesHeader {
    header_size: usize,
    pub stream_id: StreamId,
    pub size: usize,
    pub pts: Option<u64>,
    pub dts: Option<u64>
}

impl PesHeader {
    pub fn try_new(reader: &mut Cursor<Bytes>) -> Result<PesHeader> {
        let mut header_size = 6;

        let start_code:u32 = reader.read_u24::<BigEndian>()?;
        assert_eq!(start_code, 1); //test for start code

        let stream_id = StreamId::from(reader.read_u8()?);
        let pes_length = reader.read_u16::<BigEndian>()?;

        reader.seek(SeekFrom::Current(1))?; //seek past the first part of the header

        let flags: u8 = reader.read_u8()?;
        let pts_dts_flags = (flags >> 6) & 0x03;

        let mut optional_remaining = reader.read_u8()? as usize;

        let mut pts = None;
        let mut dts = None;

        if pts_dts_flags == 2 || pts_dts_flags == 3
		{
            pts = Some(Self::read_pts(reader)?);
            optional_remaining -= 5;

            if pts_dts_flags == 3 {
                dts = Some(Self::read_pts(reader)?);
                optional_remaining -= 5;
            }
        }

        header_size += optional_remaining;

        Ok(PesHeader {
            header_size,
            stream_id,
            size: pes_length as usize,
            pts,
            dts,
        })
    }

    fn read_pts(reader: &mut Cursor<Bytes>)-> Result<u64>{
        let mut pts: u64 = 0;

        // TODO: Refactor to remove using this
        #[allow(unused_assignments)]
        let mut val: u16 = 0;

        val = reader.read_u8()? as u16;
        pts |= ((val as u64 >> 1) & 0x07) << 30;
    
        val = reader.read_u16::<BigEndian>()?;
        pts |= ((val as u64 >> 1) & 0x7fff) << 15;
    
        val = reader.read_u16::<BigEndian>()?;
        pts |= (val as u64 >> 1) & 0x7fff;
    
        Ok(pts)
    }
}


#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use std::io::Cursor;
    use crate::section::pes_header::PesHeader;

    #[test]
    fn decodes_pts() {
        let raw = vec![0x31, 0x00, 0x05, 0x32, 0x81];
        let mut bytes = Cursor::new(Bytes::from(raw));

        let pts = PesHeader::read_pts(&mut bytes).unwrap();
        let hz  = 90000;
        let rem = pts % hz;
        let secs = pts / hz;
        let nsecs = 1000000000 * rem / hz;

        let time = format!("{secs}.{nsecs:09}");


        assert_eq!(pts, 72000);
        assert_eq!(time, "0.800000000");
    }

    #[test]
    fn decodes_dts() {
        let raw = vec![0x11, 0x00, 0x05, 0x1b, 0x11];
        let mut bytes = Cursor::new(Bytes::from(raw));

        let pts = PesHeader::read_pts(&mut bytes).unwrap();
        let hz  = 90000;
        let rem = pts % hz;
        let secs = pts / hz;
        let nsecs = 1000000000 * rem / hz;

        let time = format!("{secs}.{nsecs:09}");

        assert_eq!(pts, 69000);
        assert_eq!(time, "0.766666666");
    }

}
