use bytes::Bytes;
use h264::{config::DecoderConfigurationRecord, sps::Sps};
use mp4::{types::{colr::{Colr, ColorType}, avc1::Avc1, stsd::{VisualSampleEntry, SampleEntry}, avcc::AvcC}, DynBox};
use anyhow::Result;

#[derive(Default)]
pub struct SegmentStore {
    init_video: Bytes,
    init_audio: Bytes,
}

impl SegmentStore {
    pub fn new() -> SegmentStore {
        SegmentStore {
            init_video: Bytes::new(),
            init_audio: Bytes::new(),
        }
    }

    pub fn init_video(&self, config: DecoderConfigurationRecord) -> Result<(DynBox, Sps)> {

        let sps_data= config.sps[0];
        let sps = h264::sps::Sps::parse(&sps_data.payload())?;

        let colr = sps.color_config.as_ref().map(|color_config| {
            Colr::new(ColorType::Nclx {
                color_primaries: color_config.color_primaries as u16,
                matrix_coefficients: color_config.matrix_coefficients as u16,
                transfer_characteristics: color_config.transfer_characteristics as u16,
                full_range_flag: color_config.full_range,
            })
        });

        Ok((
            Avc1::new(
                SampleEntry::new(VisualSampleEntry::new(
                    sps.width as u16,
                    sps.height as u16,
                    colr,
                )),
                AvcC::new(config.clone()),
                None,
            )
            .into(),
            sps,
        ))
    }

}
