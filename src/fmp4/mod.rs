use std::{sync::Arc, cmp::max};
use crate::{session::{ManagerHandle, trigger_channel, ChannelMessage, Watcher, Message, Codec}, hls::{SegmentStores, segment_store::SegmentStore}, Opt};
use anyhow::Result;
use bytes::{Bytes, BytesMut, BufMut};
use bytesio::bytes_writer::BytesWriter;
use h264::{H264Coder, nal};
use mp4::{types::{trun::Trun, moof::Moof, mfhd::Mfhd, traf::Traf, tfhd::Tfhd, tfdt::Tfdt, mdat::Mdat, mvex::Mvex, trex::Trex, stsz::Stsz, vmhd::Vmhd, stco::Stco, stsc::Stsc, stts::Stts, stsd::Stsd, stbl::Stbl, minf::Minf, hdlr::{Hdlr, HandlerType}, mdia::Mdia, tkhd::Tkhd, trak::Trak, mvhd::Mvhd, moov::Moov, ftyp::{FourCC, Ftyp}, mdhd::Mdhd}, BoxType, DynBox};
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

    partial_begin_timestamp: Option<u32>,
    part_duration: f32,
    initialization_segment_dispatched: bool,

    h264_coder: H264Coder,

    next_h264: Option<(bool, Vec<Vec<u8>>, u64, u64, OffsetDateTime)>,
    current_h264: Option<(bool, Vec<Vec<u8>>, u64, u64, OffsetDateTime)>,
}

impl Mp4fWriter {
    fn new(opt: &Opt, stream_name: String, watcher: Watcher, stores: SegmentStores) -> Self {
        Self {
            stream_name,
            watcher,
            stores,

            latest_pcr_value: None,
            latest_pcr_timestamp_90khz: 0,
            latest_pcr_datetime: None,

            partial_begin_timestamp: None,
            part_duration: opt.part_duration,
            initialization_segment_dispatched: false,

            h264_coder: H264Coder::new(),
            next_h264: None,
            current_h264: None,
        }
    }

    async fn run(&mut self) -> Result<()> {
        while let Ok(packet) = self.watcher.recv().await {
            match packet {
                Message::ClockRef(pcr) => {
                    self.handle_pcr(pcr).await?;
                },
                Message::Packet(packet) => {
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
                },

                Message::Disconnect => break,
            }
        }
        Ok(())
    }

    async fn handle_pcr(&mut self, pcr: u64) ->Result<()> {
        let prc_value: i64 = (pcr as i64 - mpegts::HZ as i64 + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;

        let mut pcr_diff = 0;

        if let Some(latest_pcr_value) = self.latest_pcr_value {
            pcr_diff = (prc_value - latest_pcr_value + mpegts::PCR_CYCLE as i64) % mpegts::PCR_CYCLE as i64;
        }

        self.latest_pcr_timestamp_90khz += pcr_diff as u64;
            
        if let Some(latest_pcr_datetime) = self.latest_pcr_datetime {
            self.latest_pcr_datetime = Some(latest_pcr_datetime + Duration::seconds_f64(pcr_diff as f64 / mpegts::HZ as f64))
        } else {
            self.latest_pcr_datetime = Some(OffsetDateTime::now_utc() - Duration::SECOND);
        }
        
        self.latest_pcr_value = Some(prc_value); 
        Ok(())
    }

    async fn handle_audio(&mut self, _data: Bytes, _pts: u64) ->Result<()> {
        Ok(())
    }

    async fn handle_video(&mut self, data: Bytes, pts: u64, dts: Option<u64>) ->Result<()> {
        if self.latest_pcr_value.is_none() {
            return Ok(());
        }

        if self.latest_pcr_datetime.is_none() {
            return Ok(());
        }

        let dts: u64 = match dts {
            Some(dts) => dts,
            None => pts,
        };

        let latest_pcr_value = self.latest_pcr_value.unwrap();
        let latest_pcr_datetime = self.latest_pcr_datetime.unwrap();
        let cts: u64 = (pts as u64 - dts + mpegts::PCR_CYCLE as u64) % mpegts::PCR_CYCLE as u64;

        let timestamp: u64 = ((dts as i64 - latest_pcr_value as i64 + mpegts::PCR_CYCLE as i64) as u64 % mpegts::PCR_CYCLE as u64) + self.latest_pcr_timestamp_90khz as u64;

        let program_date_time = latest_pcr_datetime + Duration::seconds_f64(((dts as f64 - latest_pcr_value as f64 + mpegts::PCR_CYCLE as f64) % mpegts::PCR_CYCLE as f64) / mpegts::HZ as f64);
 
        let mut samples:Vec<Vec<u8>> = Vec::new();
        let mut keyframe_in_samples = false;

        match self.h264_coder.read_format(h264::AnnexB, &data)? {
            Some(avc) => {
                let nalus: Vec<nal::Unit> = avc.into();
                for nalu in nalus {
                    use nal::UnitType::*;
                    match &nalu.kind {
                        IdrPicture => {
                            keyframe_in_samples = true;
                            samples.push(nalu.into());
                        },
                        NonIdrPicture => {
                            samples.push(nalu.into());
                        },
                        _ => continue,
                    }
                }
            },
            None => {},
        };

        if samples.len() == 0 {
            self.next_h264 = None;
        } else {
            self.next_h264 = Some((keyframe_in_samples, samples, timestamp, cts, program_date_time));
        }

        let mut has_idr = false;
        let mut begin_timestamp: Option<u64> = None;
        let mut begin_program_date_time: Option<OffsetDateTime> = None;
        let mut writer = BytesWriter::default();

        if let Some((has_key_frame, samples, dts, cts, pdt)) = self.current_h264.clone() {
            has_idr = has_key_frame;
            begin_timestamp = Some(dts);
            begin_program_date_time = Some(pdt);
            let duration = timestamp - dts;

            //re-package to ebsp
            let mut content = BytesMut::new();

            for sample in samples {
                content.put_u32(sample.len() as u32);
                content.extend(sample);
            }
            
            let content = content.freeze();

            let mut traf = Traf::new(
                Tfhd::new(1, None, None, Some(duration as u32), None, None),
                Some(Tfdt::new(dts as u64)),
                Some(Trun::new(vec![codec::h264::trun_sample(has_idr, cts as u32, duration as u32, &content)?], None)),
            );

            traf.optimize();
            
            let mut moof = Moof::new(Mfhd::new(0), vec![traf]);
            let moof_size = moof.size();

            let traf = moof
                .traf
                .get_mut(0)
                .expect("we just created the moof with a traf");

            let trun = traf
                .trun
                .as_mut()
                .expect("we just created the video traf with a trun");

            // So the video offset will be the size of the moof + 8 bytes for the mdat header.
            trun.data_offset = Some(moof_size as i32 + 8);
            moof.mux(&mut writer)?;

            Mdat::new(vec![content]).mux(&mut writer)?;
        }

        (self.next_h264, self.current_h264) = (self.current_h264.clone(), self.next_h264.clone());

        if has_idr && !self.initialization_segment_dispatched {
            if let Some(idr) = &self.h264_coder.dcr {
                let video_config = idr.clone();

                let width = video_config.width;
                let height = video_config.height;

                let entry = codec::h264::stsd_entry(video_config)?;
    
                self.write_init_sgment(entry, width, height).await?;
            }

            self.initialization_segment_dispatched = true;
        }

        if begin_timestamp.is_none(){
            return Ok(())
        }

        self.proccess_segments(has_idr, begin_timestamp.unwrap() as u32, begin_program_date_time.unwrap()).await?;

        let mut lock = self.stores.write().await;

        if let Some(store) = lock.get_mut(&self.stream_name) {
            store.push(writer.dispose());
        }

        Ok(())
    }

    async fn proccess_segments(&mut self, has_keyframe: bool, begin_timestamp: u32, program_date_time: OffsetDateTime) -> Result<()> {
        let mut lock = self.stores.write().await;

        if let Some(store) = lock.get_mut(&self.stream_name) {
            if has_keyframe {
                if self.partial_begin_timestamp.is_some() {
                    let part_diff = begin_timestamp - self.partial_begin_timestamp.unwrap();
    
                    if ((self.part_duration * mpegts::HZ as f32).floor() as u32) < part_diff {
                        let part_duration = (self.part_duration as f32 * mpegts::HZ as f32).floor() as u32;
                        let partial_begin_timestamp = begin_timestamp - max(0, part_diff - part_duration);
                        self.partial_begin_timestamp = Some(partial_begin_timestamp);
                        store.continuous_partial(partial_begin_timestamp, false)?;
                    }
                }
                
                self.partial_begin_timestamp = Some(begin_timestamp);
                store.continuous_segment(begin_timestamp, true, program_date_time)?;
            } else if self.partial_begin_timestamp.is_some() {
                let part_diff = begin_timestamp - self.partial_begin_timestamp.unwrap();
                if (self.part_duration * mpegts::HZ as f32).floor() as u32 <= part_diff {
                    let part_duration = (self.part_duration as f32 * mpegts::HZ as f32).floor() as u32;
                    let partial_begin_timestamp = begin_timestamp - max(0, part_diff - part_duration);

                    self.partial_begin_timestamp = Some(partial_begin_timestamp);
                    store.continuous_partial(partial_begin_timestamp, false)?;
                }
            }
        }
        Ok(())
    }

    async fn write_init_sgment(&mut self, stsd_entry :DynBox, width: u32, height: u32 ) -> Result<()> {
        let mut writer: BytesWriter = BytesWriter::default();
        let compatiable_brands = vec![FourCC::Isom, FourCC::Avc1];

        Ftyp::new(FourCC::Isom, 1, compatiable_brands.clone()).mux(&mut writer)?;
        Moov::new(
            Mvhd::new(0, 0, mpegts::HZ as u32, 0, 1),
            vec![
                Trak::new(
                    Tkhd::new(0, 0, 1, 0, Some((width, height))),
                    None,
                    Mdia::new(
                        Mdhd::new(0, 0, mpegts::HZ as u32, 0),
                        Hdlr::new(HandlerType::Vide, "VideoHandler".to_string()),
                        Minf::new(
                            Stbl::new(
                                Stsd::new(vec![stsd_entry]),
                                Stts::new(vec![]),
                                Stsc::new(vec![]),
                                Stco::new(vec![]),
                                Some(Stsz::new(0, vec![])),
                            ),
                            Some(Vmhd::new()),
                            None,
                        ),
                    ),
                ),
            ],
            Some(Mvex::new(vec![Trex::new(1)], None)),
        )
        .mux(&mut writer)?;

        let mut lock = self.stores.write().await;
        if let Some(store) = lock.get_mut(&self.stream_name) {
            store.set_init_segment(writer.dispose())?;
        }

        Ok(())
    }

}

pub struct Service {
    manager_handle: ManagerHandle,
    opt: Opt,
}


impl Service {
    pub fn new(manager_handle: ManagerHandle, opt: Opt) -> Self {
        Self { manager_handle, opt }
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
                    log::info!("new_stream_store:{}, part_duration:{}, window_size:{}", stream_name, self.opt.part_duration, self.opt.window_size);
                    let store = SegmentStore::new(&self.opt);
                    lock.insert(stream_name.clone(), store);

                }
            }

            let mut fmp4_writer = Mp4fWriter::new(&self.opt, stream_name, watcher, Arc::clone(&stores));
            tokio::spawn(async move { fmp4_writer.run().await.unwrap() });
        }
        
        Ok(())
    }
}