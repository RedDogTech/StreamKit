use anyhow::Result;
use aac::AudioSpecificConfig;
use bytes::Bytes;
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
    },
    DynBox,
};

pub fn stsd_entry(data: Bytes) -> Result<(DynBox, AudioSpecificConfig)> {
    let aac_config = aac::AudioSpecificConfig::parse(data)?;

    Ok((
        Mp4a::new(
            SampleEntry::new(AudioSampleEntry::new(
                aac_config.channel_configuration.into(),
                16,
                aac_config.sampling_frequency,
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
                        data: aac_config.data.clone(),
                    }),
                )),
                None,
            )),
            None,
        )
        .into(),
        aac_config,
    ))
}
