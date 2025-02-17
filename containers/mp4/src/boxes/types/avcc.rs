use std::io;

use bytes::Bytes;
use h264::config::DecoderConfigurationRecord;

use crate::boxes::{header::BoxHeader, traits::BoxType};

#[derive(Debug, Clone, PartialEq)]
/// AVC Configuration Box
/// ISO/IEC 14496-15:2022(E) - 5.4.2
pub struct AvcC {
    pub header: BoxHeader,
    pub avc_decoder_configuration_record: DecoderConfigurationRecord,
}

impl AvcC {
    pub fn new(avc_decoder_configuration_record: DecoderConfigurationRecord) -> Self {
        Self {
            header: BoxHeader::new(Self::NAME),
            avc_decoder_configuration_record,
        }
    }
}

impl BoxType for AvcC {
    const NAME: [u8; 4] = *b"avcC";

    fn demux(header: BoxHeader, data: Bytes) -> io::Result<Self> {
        let mut reader = io::Cursor::new(data);
        Ok(Self {
            header,
            avc_decoder_configuration_record: DecoderConfigurationRecord::demux(&mut reader)?,
        })
    }

    fn primitive_size(&self) -> u64 {
        self.avc_decoder_configuration_record.size() + 2
    }

    fn primitive_mux<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        self.avc_decoder_configuration_record.mux(writer)
    }
}
