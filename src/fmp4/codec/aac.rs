use aac::aac_codec::RawAacStreamCodec;
use bytes::Bytes;
use anyhow::Result;

use mp4::{
    types::{
        esds::{
            descriptor::{
                header::DescriptorHeader,
                traits::DescriptorType,
                types::{
                    decoder_config::DecoderConfigDescriptor,
                    decoder_specific_info::DecoderSpecificInfoDescriptor, es::EsDescriptor,
                },
            },
            Esds,
        },
        mp4a::Mp4a,
        stsd::{AudioSampleEntry, SampleEntry},
        trun::{TrunSample, TrunSampleFlag},
    },
    DynBox,
};

pub fn stsd_entry(
    codec: RawAacStreamCodec,
) -> Result<DynBox> {

    Ok(
        Mp4a::new(
            SampleEntry::new(AudioSampleEntry::new(
                codec.channel_configuration.into(),
                16,
                codec.sampling_frequency_index.to_freq(),
            )),
            Esds::new(EsDescriptor::new(
                2,
                0,
                Some(0),
                None,
                Some(0),
                Some(DecoderConfigDescriptor::new(
                    0x40, // aac
                    0x05, // audio stream
                    0,    // max bitrate
                    0,    // avg bitrate
                    Some(DecoderSpecificInfoDescriptor {
                        header: DescriptorHeader::new(DecoderSpecificInfoDescriptor::TAG),
                        data: codec.audio_specific_config,
                    }),
                )),
                None,
            )),
            None,
        )
        .into(),
    )
}

pub fn trun_sample(data: &Bytes) -> Result<TrunSample> {
    Ok(
        TrunSample {
            duration: Some(1024),
            composition_time_offset: None,
            flags: Some(TrunSampleFlag {
                reserved: 0,
                is_leading: 0,
                sample_degradation_priority: 0,
                sample_depends_on: 2,
                sample_has_redundancy: 0,
                sample_is_depended_on: 0,
                sample_is_non_sync_sample: false,
                sample_padding_value: 0,
            }),
            size: Some(data.len() as u32),
        }
    )
}