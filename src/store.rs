use std::sync::Arc;
use tokio::sync::RwLock;

//use mp4::{types::{ftyp::{FourCC, Ftyp}, moov::Moov, mvhd::Mvhd, trex::Trex, mvex::Mvex, trak::Trak, tkhd::Tkhd, mdia::Mdia, stsz::Stsz, smhd::Smhd, stco::Stco, stsc::Stsc, stts::Stts, stsd::Stsd, stbl::Stbl, hdlr::{HandlerType, Hdlr}, mdhd::Mdhd, minf::Minf}, BoxType};

//const TS_HZ:u32 = 90000;

// #[derive(Default)]
// pub struct Store {
//     stores: Arc<RwLock<HashMap<String, Bytes>>>,
// }


#[derive(Clone)]
pub struct SessionStore {
    data: Arc<RwLock<String>>,
}


// #[derive(Clone)]
// pub struct Store {
//     manifest: M3u8
// }

// impl Store {
//     pub fn new() -> Store {
//         Store {
//             manifest: M3u8::new()
//         }
//     }

//     pub fn init(self) -> Result<()> {
//         let mut writer = BytesWriter::default();
        
//         // Ftyp::new(FourCC::Iso6, 0, vec![FourCC::Iso6, FourCC::Mp41]).mux(&mut writer)?;
//         // Moov::new(
//         //     Mvhd::new(0, 0, TS_HZ, 0, 1),

//         //     vec![
//         //         Trak::new(
//         //             Tkhd::new(0, 0, 2, 0, None),
//         //             None,
//         //             Mdia::new(
//         //                 Mdhd::new(0, 0, audio_sample_rate, 0),
//         //                 Hdlr::new(HandlerType::Soun, "SoundHandler".to_string()),
//         //                 Minf::new(
//         //                     Stbl::new(
//         //                         Stsd::new(vec![audio_stsd_entry]),
//         //                         Stts::new(vec![]),
//         //                         Stsc::new(vec![]),
//         //                         Stco::new(vec![]),
//         //                         Some(Stsz::new(0, vec![])),
//         //                     ),
//         //                     None,
//         //                     Some(Smhd::new()),
//         //                 ),
//         //             ),
//         //         ),
//         //     ],

//         //     Some(Mvex::new(vec![Trex::new(1)], None)),
//         // ).mux(&mut writer)?;

//         Ok(())
//     }
// }
