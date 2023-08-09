use std::{collections::VecDeque, sync::Arc};
use anyhow::Result;
use bytes::{BytesMut, Bytes};
use tokio::sync::RwLock;
use std::fmt::Write;

#[derive(Clone)]
pub struct PartialSegment {
    duration: i64,
    name: String,
    is_complete: bool,
    independant: bool,
}

#[derive(Clone)]
pub struct Segment {
    seq: u32,
    duration: i64,
    is_complete: bool,
    program_datetime: String,

    // LLHLS partial segments
    partials: Vec<PartialSegment>,
}

impl Segment {
    pub fn new(
        seq: u32,
        duration: i64,
        is_complete: bool,
        program_datetime: String,
    ) -> Self {
        Self {
            seq,
            duration,
            is_complete,
            partials: vec![],
            program_datetime,
        }
    }

    pub fn set_complete(&mut self, duration: i64) {
        self.duration = duration;
        self.is_complete = true;
    }

    pub fn add_partial(&mut self, seg: PartialSegment) {
        self.partials.push(seg);
    }
}

#[derive(Clone)]
pub struct M3u8 {
    version: u16,
    low_latency_mode: bool,
    part_target: i64,
    duration: i64,
    sequence_no: Arc<RwLock<u64>>,
    segments: VecDeque<Segment>,
    is_live: bool,
    live_ts_count: usize,
    has_init: bool,
    init_segment: Bytes,
}

impl M3u8 {
    pub fn new() -> Self {

        Self {
            version: 9,
            low_latency_mode: true,
            part_target: 1,
            duration: 10,
            sequence_no: Arc::new(RwLock::new(0)),
            segments: VecDeque::new(),
            is_live: true,
            live_ts_count: 3,
            has_init: false,
            init_segment: Bytes::new(),
        }
    }

    pub async fn get_manifest_text(&self) -> Result<String> {
        let mut manifest = String::new();

        writeln!(manifest, "#EXTM3U")?;
        writeln!(manifest, "#EXT-X-VERSION:{}", self.version)?;
        writeln!(manifest, "#EXT-X-TARGETDURATION:{}", self.duration)?;

        if self.low_latency_mode {
            writeln!(manifest, "#EXT-X-PART-INF:PART-TARGET={}", self.part_target)?;
            writeln!(manifest, "#EXT-X-SERVER-CONTROL:CAN-BLOCK-RELOAD=YES,PART-HOLD-BACK={}", self.part_target * (3.5 as i64))?;
        }

        if !self.is_live {
            writeln!(manifest, "#EXT-X-PLAYLIST-TYPE:VOD")?;
            writeln!(manifest, "#EXT-X-ALLOW-CACHE:YES")?;
        } else {
            writeln!(manifest, "#EXT-X-PLAYLIST-TYPE:EVENT")?;
        }

        if self.has_init {
            writeln!(manifest, "#EXT-X-MAP:URI=\"init\"")?;
        }

        writeln!(manifest, "#EXT-X-MEDIA-SEQUENCE:{}", self.sequence_no.read().await)?;

        for segment in &self.segments {
            let mut extinf = 0;

            writeln!(manifest, "")?; //Blank new line
            writeln!(manifest, "#EXT-X-PROGRAM-DATE-TIME:{}", segment.program_datetime)?;
            if self.low_latency_mode {
                
                for (index, partial) in segment.partials.iter().enumerate() {
                    let mut independant = String::new();

                    if partial.independant {
                        write!(independant, ",INDEPENDENT=YES")?;
                    }
                   
                    if !partial.is_complete {
                        writeln!(manifest, "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part?msn={}&part.ts={}\"{}",  segment.seq, index, independant)?;
                    } else {
                        writeln!(manifest, "#EXT-X-PART:DURATION={},URI=\"part?msn={}&part={}\"{}", partial.duration ,segment.seq, index, independant)?;
                        extinf += partial.duration;
                    }
                }
            } else {
                extinf = segment.duration;
            }

            if segment.is_complete {
                writeln!(manifest, "#EXTINF:{}", extinf)?;
                writeln!(manifest, "segment?msn={}", segment.seq)?;
            }
        }
        Ok(manifest)
    }

    pub async fn add_segment(&mut self, duration: i64, data: BytesMut) -> Result<()> {
        let segment_count = self.segments.len();
       
        if self.is_live && segment_count >= self.live_ts_count {
            let segment = self.segments.pop_front().unwrap();
            let mut s = self.sequence_no.write().await;
            *s += 1;
        }

        self.duration = std::cmp::max(duration, self.duration);

        self.segments.back_mut().unwrap().set_complete(duration);

        Ok(())
    }

    pub fn add_partial_segment(
        &mut self,
        duration: i64,
        ts_data: BytesMut,
        independent: bool,
    ) -> Result<()> {


        Ok(())
    }


}