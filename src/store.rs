use anyhow::Result;
use bytesio::bytes_writer::BytesWriter;
use mp4::{types::{ftyp::{FourCC, Ftyp}, moov::Moov, mvhd::Mvhd, trex::Trex, mvex::Mvex, trak::Trak, tkhd::Tkhd, mdia::Mdia, stsz::Stsz, smhd::Smhd, stco::Stco, stsc::Stsc, stts::Stts, stsd::Stsd, stbl::Stbl, hdlr::{HandlerType, Hdlr}, mdhd::Mdhd, minf::Minf}, BoxType};

const TS_HZ:u32 = 90000;

#[derive(Clone)]
pub struct Store {
    
}

impl Store {
    pub fn new() -> Store {
        Store {
            
        }
    }

    pub fn init(self) -> Result<()> {
        let mut writer = BytesWriter::default();
        let mut compatiable_brands = vec![FourCC::Iso5, FourCC::Iso6];

        compatiable_brands.push(FourCC::Mp41);
        
        Ftyp::new(FourCC::Iso5, 512, compatiable_brands).mux(&mut writer)?;
        Moov::new(
            Mvhd::new(0, 0, TS_HZ, 0, 1),

            vec![
                Trak::new(
                    Tkhd::new(0, 0, 2, 0, None),
                    None,
                    Mdia::new(
                        Mdhd::new(0, 0, audio_sample_rate, 0),
                        Hdlr::new(HandlerType::Soun, "SoundHandler".to_string()),
                        Minf::new(
                            Stbl::new(
                                Stsd::new(vec![audio_stsd_entry]),
                                Stts::new(vec![]),
                                Stsc::new(vec![]),
                                Stco::new(vec![]),
                                Some(Stsz::new(0, vec![])),
                            ),
                            None,
                            Some(Smhd::new()),
                        ),
                    ),
                ),
            ],

            Some(Mvex::new(vec![Trex::new(1)], None)),
        ).mux(&mut writer)?;

        Ok(())
    }
}
