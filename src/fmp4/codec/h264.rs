use bytes::Bytes;
use h264::{config::DecoderConfigurationRecord, sps::Sps};
use mp4::{DynBox, types::{colr::{ColorType, Colr}, stsd::{VisualSampleEntry, SampleEntry}, avc1::Avc1, avcc::AvcC, trun::{TrunSampleFlag, TrunSample}}};
use anyhow::{Result, bail};

pub fn stsd_entry(config: DecoderConfigurationRecord) -> Result<(DynBox, Sps)> {
    if config.sps.is_empty() {
        bail!("No H264 SPS data found");
    }

    let sps_data= config.sps[0].clone();
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
            AvcC::new(config),
            None,
        )
        .into(),
        sps,
    ))
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