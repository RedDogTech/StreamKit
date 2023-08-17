use bytes::Bytes;
use h264::config::DecoderConfigurationRecord;
use mp4::{DynBox, types::{stsd::{VisualSampleEntry, SampleEntry}, avc1::Avc1, avcc::AvcC, trun::{TrunSampleFlag, TrunSample}}};
use anyhow::Result;

pub fn stsd_entry(config: DecoderConfigurationRecord) -> Result<DynBox> {
    Ok(
        Avc1::new(
            SampleEntry::new(VisualSampleEntry::new(
                config.width as u16,
                config.height as u16,
                None
            )),
            AvcC::new(config),
            None,
        )
        .into()
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