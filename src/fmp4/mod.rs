use std::{sync::Arc, cmp::max};
use crate::{session::{ManagerHandle, trigger_channel, ChannelMessage, Watcher, Message, Codec}, hls::{SegmentStores, segment_store::SegmentStore}};
use aac::AacCoder;
use anyhow::Result;
use bytes::{Bytes, BytesMut, BufMut};
use bytesio::bytes_writer::BytesWriter;
use h264::{H264Coder, config::DecoderConfigurationRecord, nal};
use mp4::{types::{trun::{TrunSample, Trun}, moof::Moof, mfhd::Mfhd, traf::Traf, tfhd::Tfhd, tfdt::Tfdt, mdat::Mdat}, BoxType};
use time::{OffsetDateTime, Duration};
use common::FormatReader;

pub mod codec;

pub struct Mp4fWriter {
    stream_name: String,
    watcher: Watcher,
    stores: SegmentStores,

    latest_pcr_value: Option<i64>,
    latest_pcr_timestamp_90khz: u64,
    latest_pcr_datetime: Option<OffsetDateTime>,
    first_pcr: bool,

    last_timestamp: u32,
    partial_begin_timestamp: Option<u32>,
    part_duration: f32,
    initialization_segment_dispatched: bool,

    h264_coder: H264Coder,
    aac_coder: AacCoder,

    video_config: Option<DecoderConfigurationRecord>,
}

impl Mp4fWriter {
    fn new(stream_name: String, watcher: Watcher, stores: SegmentStores) -> Self {
        Self {
            stream_name,
            watcher,
            stores,

            first_pcr: false,
            latest_pcr_value: None,
            latest_pcr_timestamp_90khz: 0,
            latest_pcr_datetime: None,

            last_timestamp: 0,
            partial_begin_timestamp: None,
            part_duration: 0.1,
            initialization_segment_dispatched: false,

            h264_coder: H264Coder::new(),
            aac_coder: AacCoder::new(),

            video_config: None
        }
    }

    async fn run(&mut self) -> Result<()> {
        while let Ok(packet) = self.watcher.recv().await {
            match packet {
                Message::ClockRef(pcr) => {
                    self.handle_pcr(pcr).await?;
                },
                Message::Packet(packet) => {
                    if self.first_pcr {
                        match packet.codec {
                            Codec::H264 => {
                                self.handle_video(packet.data, packet.pts, packet.dts).await?;
                            },
                            Codec::H265 => {
                                
                            },
                            Codec::AAC => {
                                self.handle_audio(packet.data, packet.pts).await?;
                            },
                        }
                    }
                },

                Message::Disconnect => break,
            }
        }
        Ok(())
    }

    async fn handle_pcr(&mut self, pcr: u64) ->Result<()> {
        let prc_value: i64 = (pcr as i64 - mpegts::HZ as i64 + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
        
        if let Some(latest_pcr_value) = self.latest_pcr_value {
            let pcr_diff = (prc_value - latest_pcr_value + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
            self.latest_pcr_timestamp_90khz += pcr_diff as u64;
            

            if let Some(latest_pcr_datetime) = self.latest_pcr_datetime {
                self.latest_pcr_datetime = Some(latest_pcr_datetime + Duration::seconds_f64(pcr_diff as f64 / mpegts::HZ as f64))
            } else {
                self.latest_pcr_datetime = Some(OffsetDateTime::now_utc());
            }
        }
        
        self.first_pcr = true;
        self.latest_pcr_value = Some(prc_value); 
        Ok(())
    }

    async fn handle_audio(&mut self, data: Bytes, pts: u64) ->Result<()> {
        Ok(())
    }

    async fn handle_video(&mut self, data: Bytes, pts: u64, dts: Option<u64>) ->Result<()> {

        if self.latest_pcr_value.is_none() {
            return Ok(());
        }

        if self.latest_pcr_datetime.is_none() {
            return Ok(());
        }

        let dts: i64 = match dts {
            Some(dts) => dts as i64,
            None => pts as i64,
        };

        let latest_pcr_value = self.latest_pcr_value.unwrap();
        let latest_pcr_datetime = self.latest_pcr_datetime.unwrap();

        let cts: i64 = (pts as i64 - dts + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
        let timestamp: i64 = ((dts as i64 - latest_pcr_value + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64) + self.latest_pcr_timestamp_90khz as i64;
        let program_date_time = latest_pcr_datetime + Duration::seconds_f64(((dts as f64 - latest_pcr_value as f64 + mpegts::PCR_CYCLE as f64) % mpegts::PCR_CYCLE as f64) / mpegts::HZ as f64);
        
        let mut has_keyframe = false;
        let mut trun_samples: Vec<TrunSample> = Vec::new();
        let mut mdat = BytesMut::new();

        let begin_timestamp: u32 = timestamp as u32;
        let duration = timestamp - dts;

        match self.h264_coder.read_format(h264::AnnexB, &data)? {
            Some(avc) => {
                let nalus: Vec<nal::Unit> = avc.into();
                for nalu in nalus {
                    use nal::UnitType::*;
                    match &nalu.kind {
                        NonIdrPicture => {
                            trun_samples.push(codec::h264::trun_sample(false, cts as u32, duration as u32, &nalu.payload())?);
                            mdat.put(nalu.payload());
                        },
                        IdrPicture => {
                            has_keyframe = true;
                            trun_samples.push(codec::h264::trun_sample(true, cts as u32, duration as u32, &nalu.payload())?);
                            mdat.put(nalu.payload());
                        },
                        _ => return Ok(()),
                    }
                }
            },
            None => {},
        };

        let mut writer = BytesWriter::default();

        Moof::new(Mfhd::new(0), 
        vec![Traf::new(
                Tfhd::new(1, None, None, Some(duration as u32), None, None),
                Some(Tfdt::new(timestamp as u64)),
                Some(Trun::new(trun_samples, None)),
            )]
        ).mux(&mut writer)?;

        Mdat::new(vec![mdat.freeze()]).mux(&mut writer)?;


        if has_keyframe && !self.initialization_segment_dispatched {
            if let Some(idr) = &self.h264_coder.dcr {
                let video_config = idr.clone();
                self.video_config = Some(video_config.clone());
                let (entry, sps) = codec::h264::stsd_entry(video_config)?;
    
                println!("{:?}", entry);
            }

            self.initialization_segment_dispatched = true;
        }

        self.proccess_segments(has_keyframe, begin_timestamp, program_date_time);
        self.last_timestamp = timestamp as u32;

        Ok(())
    }

    fn proccess_segments(&mut self, has_keyframe: bool, begin_timestamp: u32, program_date_time: OffsetDateTime) {
        if has_keyframe {
            if self.partial_begin_timestamp.is_some() {
                let part_diff = begin_timestamp - self.partial_begin_timestamp.unwrap();

                if (self.part_duration * mpegts::HZ as f32) < part_diff as f32 {
                    self.partial_begin_timestamp = Some(begin_timestamp - max(0, (part_diff as f32 - self.part_duration as f32 * mpegts::HZ as f32) as u32));
                    println!("m3u8.continuousPartial({:?}, False)",  self.partial_begin_timestamp );
                }
            }

            self.partial_begin_timestamp = Some(begin_timestamp);
            println!("m3u8.continuousSegment({:?}, True, {}", self.partial_begin_timestamp, program_date_time);
        } else if self.partial_begin_timestamp.is_some() {
            let part_diff = begin_timestamp - self.partial_begin_timestamp.unwrap();

            if (self.part_duration * mpegts::HZ as f32) <= part_diff as f32 {
                self.partial_begin_timestamp = Some(begin_timestamp - max(0, (part_diff as f32 - self.part_duration as f32 * mpegts::HZ as f32) as u32));
                println!("m3u8.continuousPartial({:?})", self.partial_begin_timestamp);
            }
        }
    }

}

pub struct Service {
    manager_handle: ManagerHandle,
}


impl Service {
    pub fn new(manager_handle: ManagerHandle) -> Self {
        Self { manager_handle }
    }

    pub async fn run(self, stores: SegmentStores)-> Result<()> {        
        let (trigger, mut trigger_handle) = trigger_channel();

        if let Err(_) = self
            .manager_handle
            .send(ChannelMessage::RegisterTrigger("create_session", trigger))
        {
            log::error!("Failed to register session trigger");
            return Ok(());
        }

        while let Some((stream_name, watcher)) = trigger_handle.recv().await {
            let mut lock = stores.write().await;
            match lock.get_mut(&stream_name) {
                Some(_) => {
                    log::warn!("duplicate stream store {}", stream_name);
                }

                None => {
                    log::info!("creating new stream store {}", stream_name);
                    let store = SegmentStore::new();
                    lock.insert(stream_name.clone(), store);

                }
            }

            let mut fmp4_writer = Mp4fWriter::new(stream_name, watcher, Arc::clone(&stores));
            tokio::spawn(async move { fmp4_writer.run().await.unwrap() });
        }
        
        Ok(())
    }
}