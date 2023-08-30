use anyhow::Result;
use bytes::Bytes;
use h265::config::HEVCDecoderConfigurationRecord;
use mp4::{types::{hev1::Hev1, stsd::{VisualSampleEntry, SampleEntry}, hvcc::HvcC, colr::{Colr, ColorType}, trun::{TrunSample, TrunSampleFlag}}, DynBox};

pub fn stsd_entry(config: HEVCDecoderConfigurationRecord) -> Result<DynBox> {

    
    // let colr = sps.color_config.as_ref().map(|color_config| {
    //     Colr::new(ColorType::Nclx {
    //         color_primaries: color_config.color_primaries as u16,
    //         matrix_coefficients: color_config.matrix_coefficients as u16,
    //         transfer_characteristics: color_config.transfer_characteristics as u16,
    //         full_range_flag: color_config.full_range,
    //     })
    // });

    Ok(
        Hev1::new(
            SampleEntry::new(VisualSampleEntry::new(
                1280 as u16,
                720 as u16,
                None,
            )),
            HvcC::new(config),
            None,
        ).into()
    )
}

pub fn trun_sample(
    keyframe: bool,
    composition_time_offset: u32,
    duration: u32,
    data: &Bytes,
) -> Result<TrunSample> {
    Ok(TrunSample {
        composition_time_offset: Some(composition_time_offset as i64),
        duration: Some(duration),
        flags: Some(TrunSampleFlag {
            reserved: 0,
            is_leading: 0,
            sample_degradation_priority: 0,
            sample_depends_on: if keyframe {
                2
            } else {
                1
            },
            sample_has_redundancy: 0,
            sample_is_depended_on: 0,
            sample_is_non_sync_sample: keyframe,
            sample_padding_value: 0,
        }),
        size: Some(data.len() as u32),
    })
}