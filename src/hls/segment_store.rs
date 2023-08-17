use std::{cmp::max, collections::VecDeque};
use std::fmt::Write;
use bytes::{Bytes, BytesMut, BufMut};
use anyhow::Result;
use time::{OffsetDateTime, Duration, format_description::well_known::Rfc3339};

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

    fn extinf(&self) -> Option<Duration> {
        if let Some(end_pts) = self.end_pts {
            return Some(Duration::seconds_f32((end_pts as f32 - self.begin_pts as f32 + mpegts::PCR_CYCLE as f32) %  mpegts::PCR_CYCLE as f32) /  mpegts::HZ);
        }

        None
    }
}

#[derive(Debug)]
struct Segment {
    partials: VecDeque<PartialSegment>,
    begin_pts: u32,
    end_pts: Option<u32>,
    key_frame: bool,
    program_datetime: OffsetDateTime,
    data: BytesMut,
    test: usize,
}

impl Segment {
    fn new(begin_pts: u32, key_frame: bool, program_datetime: OffsetDateTime, test: usize) -> Self {
        let mut partials = VecDeque::new();
        partials.push_front(PartialSegment::new(begin_pts, key_frame));

        Self {
            begin_pts,
            end_pts: None,
            key_frame,
            program_datetime,
            partials,
            data: BytesMut::new(),
            test,
        }
    }

    pub fn complete(&mut self, end_pts: u32) {
        self.end_pts = Some(end_pts)
    }

    fn push(&mut self, data: Bytes) {
        // FIXME:   both the partials and segments store both
        //          should find a managable way of storing
        //          just a single one

        self.data.put(data.clone());

        if let Some(partial) = self.partials.front_mut() {
            partial.push(data);
        }
    }

    fn extinf(&self) -> Option<Duration> {
        if let Some(end_pts) = self.end_pts {
            return Some(Duration::seconds_f32((end_pts as f32 - self.begin_pts as f32 + mpegts::PCR_CYCLE as f32) %  mpegts::PCR_CYCLE as f32) /  mpegts::HZ);
        }

        None
    }

    fn new_partial(&mut self, begin_pts: u32, key_frame: bool) {
        self.partials.push_front(PartialSegment::new(begin_pts, key_frame));
    }

    fn payload(&self) -> Option<Bytes> {
        if self.end_pts.is_some() {
            return Some(self.data.clone().freeze());
        }

        None
    }

}

#[derive(Default)]
pub struct SegmentStore {
    init_segment: Bytes,
    media_sequence: usize,
    published: bool,
    windows_size: Option<usize>,
    target_duration: Duration,
    part_duration: f32,
    low_latency_mode: bool,
    version: usize,
    is_live: bool,
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
            target_duration: Duration::SECOND,
            part_duration: opt.part_duration,
            low_latency_mode: false,
            version: 9,
            is_live: true,
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

    pub fn continuous_partial(&mut self, end_pts: u32, key_frame: bool) {
        if let Some(last_segment) = self.segments.front_mut() {
            //println!("pushing to {:?}", last_segment.test);
            if let Some(partial) = last_segment.partials.front_mut() {
                partial.complete(end_pts);
            }
        }

        self.new_partial(end_pts, key_frame);
    }

    fn new_partial(&mut self, end_pts: u32, key_frame: bool) {
        if let Some(last_segment) = self.segments.front_mut() {
            last_segment.new_partial(end_pts, key_frame);
        }
    }

    fn new_segment(&mut self, begin_pts: u32, key_frame: bool, program_datetime: OffsetDateTime) {
        //println!("new_segment:{} ", self.media_sequence);
        self.segments.push_front(Segment::new(begin_pts, key_frame, program_datetime, self.media_sequence));

        if let Some(window_size) = self.windows_size {
            //println!("window_size:{} {}", window_size, self.segments.len());
            while window_size < self.segments.len() {
                if let Some(last_segment) = self.segments.pop_back() {
                    self.outdated.push_back(last_segment);
                }

                //println!("=====new media_sequence {}", self.media_sequence);
                self.media_sequence += 1;
            }

            while window_size < self.outdated.len() {
                self.outdated.pop_front();
            }
        }
    }

    pub fn continuous_segment(&mut self, end_pts: u32, key_frame: bool, program_datetime: OffsetDateTime) {
        if let Some(segment) = self.segments.front_mut() {
            segment.complete(end_pts);
            self.published = true;
        }

        self.new_segment(end_pts, key_frame, program_datetime);
    }

    pub fn push(&mut self, data: Bytes) {
        if let Some(segment) = self.segments.front_mut() {
            segment.push(data);
        }
    }

    fn estimated_tartget_duration(&self) -> Duration {
        let mut target_duration = self.target_duration;

        for segment in self.segments.iter() {
            if let Some(duration) = segment.extinf() {
                target_duration = max(target_duration, duration)
            }
        }

        target_duration
    }

    fn in_range(&self, msn: usize) -> bool {
        (self.media_sequence <= msn) && (msn < self.media_sequence + self.segments.len())
    }

    fn in_outdated(&self, msn: usize) -> bool {
        (self.media_sequence > msn) && (msn >= self.media_sequence - self.segments.len())
    }

    pub fn segment(&self, msn: usize) -> Option<Bytes> {
        if !self.in_range(msn) {
            if !self.in_outdated(msn) {
                return None;
            } else {
                let index = (self.media_sequence - msn) - 1;
                return self.outdated[index].payload();
            }
        }
        let index = msn - self.media_sequence;
        return self.segments[index].payload();
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

    pub async fn get_manifest_text(&self) -> Result<String> {
        let mut manifest = String::new();

        writeln!(manifest, "#EXTM3U")?;
        writeln!(manifest, "#EXT-X-VERSION:{}", self.version)?;
        writeln!(manifest, "#EXT-X-TARGETDURATION:{:.06}", self.estimated_tartget_duration().as_seconds_f32())?;

        if self.low_latency_mode {
            writeln!(manifest, "#EXT-X-PART-INF:PART-TARGET={:.06}", self.part_duration)?;
            writeln!(manifest, "#EXT-X-SERVER-CONTROL:CAN-BLOCK-RELOAD=YES,PART-HOLD-BACK={}", self.part_duration * (3.001 as f32))?;
        }

        if !self.is_live {
            writeln!(manifest, "#EXT-X-PLAYLIST-TYPE:VOD")?;
            writeln!(manifest, "#EXT-X-ALLOW-CACHE:YES")?;
        } else {
            writeln!(manifest, "#EXT-X-PLAYLIST-TYPE:EVENT")?;
        }

        if self.init_segment_ready().is_some() {
            writeln!(manifest, "#EXT-X-MAP:URI=\"init.mp4\"")?;
        }

        writeln!(manifest, "#EXT-X-MEDIA-SEQUENCE:{}", self.media_sequence)?;

        for (seq, segment) in self.segments.iter().rev().enumerate() {
            let msn = self.media_sequence + seq;
            writeln!(manifest, "")?; //Blank new line
            writeln!(manifest, "#EXT-X-PROGRAM-DATE-TIME:{}", segment.program_datetime.format(&Rfc3339)?)?;
            if self.low_latency_mode {
                
                for (index, partial) in segment.partials.iter().rev().enumerate() {
                    if seq >= self.segments.len() - 4 {
                        let mut independant = String::new();

                        if partial.is_independant() {
                            write!(independant, ",INDEPENDENT=YES")?;
                        }
                        if let Some(duration) = partial.extinf() {
                            writeln!(manifest, "#EXT-X-PART:DURATION={:.06},URI=\"part.m4s?msn={}&part={}\"{}", duration.as_seconds_f32(), msn, index, independant)?;
                        } else {
                            writeln!(manifest, "#EXT-X-PRELOAD-HINT:TYPE=PART,URI=\"part.m4s?msn={}&part={}\"{}",  msn, index, independant)?;
                        }
                    }
                }
            }

            if let Some(duration) = segment.extinf() {
                writeln!(manifest, "#EXTINF:{:.06}", duration.as_seconds_f32())?;
                writeln!(manifest, "segment.m4s?msn={}", msn)?;
            }
        }
        Ok(manifest)
    }

    pub fn set_init_segment(&mut self, data: Bytes) -> Result<()> {
        self.init_segment = data;
        Ok(())
    }

}
