use std::collections::VecDeque;
use std::fmt::Write;
use bytes::{Bytes, BytesMut, BufMut};
use anyhow::Result;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};

use crate::Opt;

#[derive(Debug)]
struct PartialSegment {
    data: BytesMut,
    begin_pts: u32,
    end_pts: Option<u32>,
    key_frame: bool,
}

impl PartialSegment {
    fn new(begin_pts: u32, key_frame: bool) -> Self {
        Self {
            data: BytesMut::new(),
            begin_pts,
            end_pts: None,
            key_frame,
        }
    }

    fn push(&mut self, data: Bytes) {
        self.data.put(data);
    }

    fn payload(&self) -> Option<Bytes> {
        if self.end_pts.is_some() {
            return Some(self.data.clone().freeze());
        }

        None
    }

    fn complete(&mut self, end_pts: u32) {
        self.end_pts = Some(end_pts);
    }

    fn is_independant(&self) -> bool {
        self.key_frame
    }

    #[inline(always)]
    fn duration(&self) -> Option<f64> {
        if let Some(end_pts) = self.end_pts {
            return Some(((end_pts as u64 - self.begin_pts as u64 + mpegts::PCR_CYCLE as u64) % mpegts::PCR_CYCLE ) as f64 / mpegts::HZ as f64);
        }
        None
    }
}

#[derive(Debug)]
struct Segment {
    partials: Vec<PartialSegment>,
    begin_pts: u32,
    end_pts: Option<u32>,
    key_frame: bool,
    program_datetime: OffsetDateTime,
    queues: Vec<UnboundedSender<Option<Bytes>>>,
    data: BytesMut,
}

impl Segment {
    fn new(begin_pts: u32, key_frame: bool, program_datetime: OffsetDateTime) -> Self {
        let mut partials = Vec::new();
        partials.push(PartialSegment::new(begin_pts, key_frame));

        Self {
            begin_pts,
            end_pts: None,
            key_frame,
            program_datetime,
            partials,
            queues: Vec::new(),
            data: BytesMut::new(),
        }
    }

    pub async fn response(&mut self) -> UnboundedReceiver<Option<Bytes>> {
        let (sender, reciver) = mpsc::unbounded_channel::<Option<Bytes>>();

        let buffer = self.data.clone().freeze();
        sender.send(Some(buffer)).ok();

        if self.is_complete() {
            sender.send(None).ok();
        } else {
            self.queues.push(sender);
        }

        reciver
    }

    fn is_complete(&self) -> bool {
        self.end_pts.is_some()
    }

    pub fn complete(&mut self, end_pts: u32) {
        self.end_pts = Some(end_pts);
        self.complete_partial(end_pts);

        for q in &self.queues {
            let _ = q.send(None);
        }
        self.queues.clear();
    }

    pub fn complete_partial(&mut self, end_pts: u32) {
        if let Some(partial) = self.partials.last_mut(){
            partial.complete(end_pts)
        }
    }

    fn push(&mut self, data: Bytes) {
        // FIXME:   both the partials and segments store both
        //          should find a managable way of storing
        //          just a single one

        self.data.put(data.clone());

        if let Some(partial) = self.partials.last_mut() {
            partial.push(data.clone());
        }

        for q in &self.queues {
            let _ = q.send(Some(data.clone()));
        }
    }

    #[inline(always)]
    fn duration(&self) -> Option<f64> {
        if let Some(end_pts) = self.end_pts {
            return Some(((end_pts as u64 - self.begin_pts as u64 + mpegts::PCR_CYCLE as u64) % mpegts::PCR_CYCLE ) as f64 / mpegts::HZ as f64);
        }
        None
    }

    fn new_partial(&mut self, begin_pts: u32, key_frame: bool) {
        self.partials.push(PartialSegment::new(begin_pts, key_frame));
    }

}

pub struct SegmentStore {
    init_segment: Bytes,
    media_sequence: usize,
    published: bool,
    windows_size: Option<usize>,
    part_duration: f32,
    low_latency_mode: bool,
    version: usize,
    is_live: bool,
    manifest_body: Option<String>,
    segments: VecDeque<Segment>,
    outdated: VecDeque<Segment>,
}

impl SegmentStore {
    pub fn new(opt: &Opt) -> SegmentStore {
        SegmentStore {
            init_segment: Bytes::new(),
            media_sequence: 0,
            published: false,
            windows_size: Some(opt.window_size),
            part_duration: opt.part_duration,
            low_latency_mode: false,
            version: 9,
            is_live: true,
            manifest_body: None,
            segments: VecDeque::new(),
            outdated: VecDeque::new()
        }
    }

    pub fn init_segment_ready(&self) -> Option<Bytes> {
        if self.init_segment.len() != 0 {
            return Some(self.init_segment.clone());
        }
        None
    }

    pub fn continuous_partial(&mut self, end_pts: u32, key_frame: bool) -> Result<()> {
        if let Some(last_segment) = self.segments.front_mut() {
            if let Some(partial) = last_segment.partials.last_mut() {
                partial.complete(end_pts);
                self.generate_manfiest()?;
            }
        }

        self.new_partial(end_pts, key_frame);
        Ok(())
    }

    fn new_partial(&mut self, end_pts: u32, key_frame: bool) {
        if let Some(last_segment) = self.segments.front_mut() {
            last_segment.new_partial(end_pts, key_frame);
        }
    }

    fn new_segment(&mut self, begin_pts: u32, key_frame: bool, program_datetime: OffsetDateTime) {
        println!("new segment {}", self.media_sequence);
        self.segments.push_front(Segment::new(begin_pts, key_frame, program_datetime));

        if let Some(window_size) = self.windows_size {
            while window_size < self.segments.len() {
                if let Some(last_segment) = self.segments.pop_back() {
                    self.outdated.push_back(last_segment);
                    self.media_sequence += 1;
                }
            }

            while window_size < self.outdated.len() {
                self.outdated.pop_front();
            }
        }
    }

    pub fn continuous_segment(&mut self, end_pts: u32, key_frame: bool, program_datetime: OffsetDateTime) -> Result<()> {
        if let Some(segment) = self.segments.front_mut() {
            self.published = true;
            segment.complete(end_pts);
            self.generate_manfiest()?;
        }

        self.new_segment(end_pts, key_frame, program_datetime);
        Ok(())
    }

    pub fn push(&mut self, data: Bytes) {
        if let Some(segment) = self.segments.front_mut() {
            segment.push(data);
        }
    }

    #[inline(always)]
    fn target_duration(&self) -> f64 {
        let mut max: f64 = 1.0;
        for segment in self.segments.iter() {
            max = max.max(segment.duration().unwrap_or(0.0));
        }
    
        return max.ceil();
    }

    fn in_range(&self, msn: usize) -> bool {
        (self.media_sequence <= msn) && (msn < self.media_sequence + self.segments.len())
    }

    fn in_outdated(&self, msn: usize) -> bool {
        (self.media_sequence > msn) && (msn >= self.media_sequence - self.segments.len())
    }

    pub async fn segment(&mut self, msn: usize) -> Option<UnboundedReceiver<Option<Bytes>>> {
        println!("segment request {} ({}), index({})", msn, self.media_sequence, (self.media_sequence - msn) as i64);
        let msn = msn + 10;

        if !self.in_range(msn) {
            if !self.in_outdated(msn) {
                return None;
            } else {
                let index = (self.media_sequence - msn) - 1;
                return Some(self.outdated[index].response().await);
            }
        }
        println!("segment request {} ({})", msn, self.media_sequence);
        let index = msn - self.media_sequence;
        println!("segment request {}", index);

        return Some(self.segments[index].response().await);
    }

    pub fn partial(&self, msn: usize, part: usize) -> Option<Bytes> {
        if !self.in_range(msn) {
            if !self.in_outdated(msn) {
                return None;
            } else {
                let index = (self.media_sequence - msn) - 1;
                if part > self.outdated[index].partials.len() {
                    return None
                } else {
                    return self.outdated[index].partials[part].payload();
                }
            }
        }

        let index = msn - self.media_sequence;
        if part > self.segments[index].partials.len() {
            return None;
        } else {
            return self.segments[index].partials[part].payload();
        }
    }

    pub async fn get_manifest_text(&self) -> Option<String> {
        if self.published {
            return self.manifest_body.clone();
        }

        None
    }

    pub fn generate_manfiest(&mut self) -> Result<()> {
        let mut manifest = String::new();

        writeln!(manifest, "#EXTM3U")?;
        writeln!(manifest, "#EXT-X-VERSION:{}", self.version)?;
        writeln!(manifest, "#EXT-X-TARGETDURATION:{:.06}", self.target_duration())?;

        if self.low_latency_mode {
            writeln!(manifest, "#EXT-X-PART-INF:PART-TARGET={:.06}", self.part_duration)?;
            writeln!(manifest, "#EXT-X-SERVER-CONTROL:CAN-BLOCK-RELOAD=YES,PART-HOLD-BACK={}", self.part_duration * (3.001 as f32))?;
        }

        if !self.is_live {
            writeln!(manifest, "#EXT-X-PLAYLIST-TYPE:VOD")?;
            writeln!(manifest, "#EXT-X-ALLOW-CACHE:YES")?;
        }

        if self.init_segment_ready().is_some() {
            writeln!(manifest, "#EXT-X-MAP:URI=\"init.mp4\"")?;
        }

        writeln!(manifest, "#EXT-X-MEDIA-SEQUENCE:{}", self.media_sequence)?;

        for (seq, segment) in self.segments.iter().enumerate() {
            let msn = self.media_sequence + seq;
            writeln!(manifest, "")?; //Blank new line
            writeln!(manifest, "#EXT-X-PROGRAM-DATE-TIME:{}", segment.program_datetime.format(&Rfc3339)?)?;
            if self.low_latency_mode {
                
                if seq >= self.segments.len() - 4 {
                    for (index, partial) in segment.partials.iter().enumerate() {   
                        let mut independant = String::new();

                        if partial.is_independant() {
                            write!(independant, ",INDEPENDENT=YES")?;
                        }
                        if let Some(duration) = partial.duration() {
                            writeln!(manifest, "#EXT-X-PART:DURATION={:.06},URI=\"part.m4s?msn={}&part={}\"{}", duration, msn, index, independant)?;
                        } else {
                            writeln!(manifest, "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.m4s?msn={}&part={}\"{}",  msn, index, independant)?;
                        }
                    }
                }
            }

            if let Some(duration) = segment.duration() {
                writeln!(manifest, "#EXTINF:{:.06}", duration)?;
                writeln!(manifest, "segment.m4s?msn={}", msn as i64 - 15)?;
            }
        }
        self.manifest_body = Some(manifest);
        Ok(())
    }

    pub fn set_init_segment(&mut self, data: Bytes) -> Result<()> {
        self.init_segment = data;
        Ok(())
    }

}
