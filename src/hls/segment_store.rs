use bytes::Bytes;
use anyhow::Result;
use bytesio::bytes_writer::BytesWriter;
use h264::config::DecoderConfigurationRecord;
use mp4::{
    types::{colr::{Colr, ColorType},
    avc1::Avc1,
    stsd::{VisualSampleEntry, SampleEntry, Stsd},
    avcc::AvcC,
    ftyp::{FourCC, Ftyp},
    mvex::Mvex, trex::Trex, moov::Moov, mvhd::Mvhd,
    trak::Trak, tkhd::Tkhd, mdia::Mdia, mdhd::Mdhd,
    hdlr::{Hdlr, HandlerType},
    minf::Minf, stbl::Stbl, stsc::Stsc, stco::Stco,
    stts::Stts, vmhd::Vmhd, stsz::Stsz}, DynBox, BoxType
};

#[derive(Default)]
pub struct SegmentStore {
    video_stsd: Option<DynBox>,
    audio_stsd: Option<DynBox>,

    compatiable_brands: Vec<FourCC>,

    init_segment: Bytes,
}

impl SegmentStore {
    pub fn new() -> SegmentStore {
        SegmentStore {
            video_stsd: None,
            audio_stsd: None,
            init_segment: Bytes::new(),
            compatiable_brands: vec![FourCC::Iso5, FourCC::Iso6],
        }
    }

    pub fn init_segment_ready(&self) -> Option<Bytes> {
        if self.init_segment.len() != 0 {
            return Some(self.init_segment.clone());
        }
        None
    }

    pub fn init_video_stsd(&mut self, config: DecoderConfigurationRecord) -> Result<()> {
        self.compatiable_brands.push(FourCC::Avc1);

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

        self.video_stsd = Some(Avc1::new(
            SampleEntry::new(VisualSampleEntry::new(
                sps.width as u16,
                sps.height as u16,
                colr,
            )),
            AvcC::new(config.clone()),
            None,
        )
        .into());

        self.init_segment()?;

        Ok(())
    }

    pub fn init_segment(&mut self) -> Result<()> {
        let mut writer = BytesWriter::default();

        Ftyp::new(FourCC::Iso5, 512, self.compatiable_brands.clone()).mux(&mut writer)?;
        Moov::new(
            Mvhd::new(0, 0, mpegts::HZ as u32, 0, 1),
            vec![
                Trak::new(
                    Tkhd::new(0, 0, 1, 0, Some((1280, 0))),
                    None,
                    Mdia::new(
                        Mdhd::new(0, 0, mpegts::HZ as u32, 0),
                        Hdlr::new(HandlerType::Vide, "VideoHandler".to_string()),
                        Minf::new(
                            Stbl::new(
                                Stsd::new(vec![self.video_stsd.clone().unwrap()]),
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
                // Trak::new(
                //     Tkhd::new(0, 0, 2, 0, None),
                //     None,
                //     Mdia::new(
                //         Mdhd::new(0, 0, audio_sample_rate, 0),
                //         Hdlr::new(HandlerType::Soun, "SoundHandler".to_string()),
                //         Minf::new(
                //             Stbl::new(
                //                 Stsd::new(vec![audio_stsd_entry]),
                //                 Stts::new(vec![]),
                //                 Stsc::new(vec![]),
                //                 Stco::new(vec![]),
                //                 Some(Stsz::new(0, vec![])),
                //             ),
                //             None,
                //             Some(Smhd::new()),
                //         ),
                //     ),
                // ),
            ],
            Some(Mvex::new(vec![Trex::new(1)], None)),
        )
        .mux(&mut writer)?;

        self.init_segment = writer.dispose();

        Ok(())
    }

}
