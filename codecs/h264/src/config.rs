use std::io::{self, Write};

use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};
use bytes::{Bytes, BytesMut, BufMut};
use bytesio::{bit_writer::BitWriter, bytes_reader::BytesCursor, bit_reader::BitReader};
use exp_golomb::{read_exp_golomb, read_signed_exp_golomb};

use {
    super::{nal, AvcError},
    bytes::Buf,
    std::{convert::TryFrom, io::Cursor},
};

// Bits | Name
// ---- | ----
// 8    | Version
// 8    | Profile Indication
// 8    | Profile Compatability
// 8    | Level Indication
// 6    | Reserved
// 2    | NALU Length
// 3    | Reserved
// 5    | SPS Count
// 16   | SPS Length
// var  | SPS
// 8    | PPS Count
// 16   | PPS Length
// var  | PPS
#[derive(Debug, Clone, PartialEq)]
pub struct DecoderConfigurationRecord {
    pub version: u8,
    pub profile_indication: u8,
    pub profile_compatability: u8,
    pub level_indication: u8,
    pub nalu_size: u8,
    pub sps: Vec<nal::Unit>,
    pub pps: Vec<nal::Unit>,
    pub color_config: Option<ColorConfig>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
/// Color config for SPS
pub struct ColorConfig {
    pub full_range: bool,
    pub color_primaries: u8,
    pub transfer_characteristics: u8,
    pub matrix_coefficients: u8,
}

#[derive(Debug, Clone, PartialEq)]
/// AVC (H.264) Extended Configuration
/// ISO/IEC 14496-15:2022(E) - 5.3.2.1.2
pub struct AvccExtendedConfig {
    pub chroma_format: u8,
    pub bit_depth_luma_minus8: u8,
    pub bit_depth_chroma_minus8: u8,
   // pub sequence_parameter_set_ext: Vec<Bytes>,
}

impl Default for DecoderConfigurationRecord {
    fn default() -> Self {
        Self {
            version: 1u8,
            profile_indication: 0u8,
            profile_compatability: 0u8,
            level_indication: 0u8,
            nalu_size: 4u8,
            sps: vec![],
            pps: vec![],
            color_config: None,
            width: 0,
            height: 0,
        }
    }
}

impl DecoderConfigurationRecord {
    pub fn mux<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        let mut bit_writer = BitWriter::default();

        bit_writer.write_u8(self.version)?;
        bit_writer.write_u8(self.profile_indication)?;
        bit_writer.write_u8(self.profile_compatability)?;
        bit_writer.write_u8(self.level_indication)?;
        bit_writer.write_u8(0xFF)?;

        bit_writer.write_u8(0b11100001)?; // sps count 1

        let sps: Vec<u8> = self.sps.first().unwrap().into();
        bit_writer.write_u16::<BigEndian>(sps.len() as u16)?;
        bit_writer.write_all(&sps)?;

        bit_writer.write_u8(1)?;
        let pps: Vec<u8> = self.pps.first().unwrap().into();
        bit_writer.write_u16::<BigEndian>(pps.len() as u16)?;
        bit_writer.write_all(&pps)?;

        writer.write_all(&bit_writer.into_inner())?;
        Ok(())
    }

    //TODO:: Remove
    pub fn demux(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        Ok(DecoderConfigurationRecord::try_from(reader.chunk()).unwrap())
    }

    pub fn size(&self) -> u64 {
        1 // configuration_version
        + 1 // avc_profile_indication
        + 1 // profile_compatibility
        + 1 // avc_level_indication
        + 1 // length_size_minus_one
        + 1
        + self.sps.iter().map(|sps| {
            2 // sps_length
            + sps.payload().len() as u64
        }).sum::<u64>() // sps
        + 1 // num_of_picture_parameter_sets
        + self.pps.iter().map(|pps| {
            2 // pps_length
            + pps.payload().len() as u64
        }).sum::<u64>() // pps
    }

    // ISO/IEC-14496-10-2022 - 3.1.48
    fn ebsp_to_rbsp(ebsp_data: Bytes) -> Bytes {
        let mut rbsp_data = BytesMut::new();
        let mut pos = 0;
    
        while pos < ebsp_data.len() {
            if pos + 2 < ebsp_data.len() && ebsp_data[pos] == 0x00 && ebsp_data[pos + 1] == 0x00 && ebsp_data[pos + 2] == 0x03 {
                // Emulation prevention byte detected, skip it
                rbsp_data.extend_from_slice(&ebsp_data[pos..pos + 2]);
                pos += 3;
            } else {
                rbsp_data.put_u8(ebsp_data[pos]);
                pos += 1;
            }
        }
    
        rbsp_data.freeze()
    }

    pub fn parse(&mut self) -> Result<(), AvcError> {

        let sps = self.sps.first().unwrap().payload();

        let buffer = DecoderConfigurationRecord::ebsp_to_rbsp(sps);

        let mut bit_reader = BitReader::from(buffer);

        self.profile_indication = bit_reader.read_u8()?;
        self.profile_compatability = bit_reader.read_u8()?;
        self.level_indication = bit_reader.read_u8()?;


        let chroma_format_idc = 1;

        read_exp_golomb(&mut bit_reader)?; // seq_parameter_set_id
        read_exp_golomb(&mut bit_reader)?; // log2_max_frame_num_minus4

        let pic_order_cnt_type = read_exp_golomb(&mut bit_reader)?;

        if pic_order_cnt_type == 0{
            read_exp_golomb(&mut bit_reader)?;
        }else if pic_order_cnt_type ==1 {
            bit_reader.seek_bits(1)?;
            read_signed_exp_golomb(&mut bit_reader)?;
            read_signed_exp_golomb(&mut bit_reader)?;
            let num_ref_frames_in_pic_order_cnt_cycle = read_exp_golomb(&mut bit_reader)?;
            for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                read_signed_exp_golomb(&mut bit_reader)?; // offset_for_ref_frame
            }
        }

        let ref_frames = read_exp_golomb(&mut bit_reader)?;
        bit_reader.seek_bits(1)?;

        let pic_width_in_mbs_minus1 = read_exp_golomb(&mut bit_reader)?;
        let pic_height_in_map_units_minus1 = read_exp_golomb(&mut bit_reader)?;

        let frame_mbs_only_flag = bit_reader.read_bit()?;

        if !frame_mbs_only_flag {
            bit_reader.read_bit()?;
        }
        bit_reader.read_bit()?;


        let mut frame_crop_right_offset = 0;
        let mut frame_crop_top_offset = 0;
        let mut frame_crop_bottom_offset = 0;
        let frame_cropping_flag = bit_reader.read_bit()?;
        let mut frame_crop_left_offset = 0;

        if frame_cropping_flag {
            frame_crop_left_offset = read_exp_golomb(&mut bit_reader)?;
            frame_crop_right_offset = read_exp_golomb(&mut bit_reader)?;
            frame_crop_top_offset = read_exp_golomb(&mut bit_reader)?;
            frame_crop_bottom_offset = read_exp_golomb(&mut bit_reader)?;
        }

        let mut sar_width = 1;
        let mut sar_height = 1;
        let mut fps = 0;
        let mut fps_fixed =true;
        let mut fps_num = 0;
        let mut fps_den = 0;
        let vui_parameters_present_flag = bit_reader.read_bit()?;

        if vui_parameters_present_flag {
             // aspect_ratio_info_present_flag
            if bit_reader.read_bit()? {
                let aspect_ratio_idc = bit_reader.read_u8()? as usize;
                let sar_w_table = [1, 12, 10, 16, 40, 24, 20, 32, 80, 18, 15, 64, 160, 4, 3, 2];
                let sar_h_table = [1, 11, 11, 11, 33, 11, 11, 11, 33, 11, 11, 33,  99, 3, 2, 1];

                if 0 < aspect_ratio_idc && aspect_ratio_idc <= 16 {

                sar_width = sar_w_table[aspect_ratio_idc - 1];
                sar_height = sar_h_table[aspect_ratio_idc - 1];

                } else if aspect_ratio_idc == 255 {
                    sar_width = bit_reader.read_bits(16)?;
                    sar_height = bit_reader.read_bits(16)?;
                }
            }

            // overscan_info_present_flag
            if bit_reader.read_bit()? {
                bit_reader.read_bit()?;
            }
            
            // video_signal_type_present_flag
            if bit_reader.read_bit()? {
                bit_reader.seek_bits(4)?;

                // colour_description_present_flag
                if bit_reader.read_bit()? {
                    bit_reader.seek_bits(24)?;
                }
            }

            // chroma_loc_info_present_flag
            if bit_reader.read_bit()? {
                read_exp_golomb(&mut bit_reader)?;
                read_exp_golomb(&mut bit_reader)?;
            }

            // timing_info_present_flag
            if bit_reader.read_bit()? {
                let num_units_in_tick = bit_reader.read_u32::<BigEndian>()?;
                let time_scale = bit_reader.read_u32::<BigEndian>()?;
                fps_fixed = bit_reader.read_bit()?;

                fps_num = time_scale;
                fps_den = num_units_in_tick * 2;
                fps = fps_num / fps_den;
            }
        }

        let mut crop_unit_x =0;
        let mut crop_unit_y = 0;

        if chroma_format_idc == 0 {
            crop_unit_x = 1;
            crop_unit_y = 2 - frame_mbs_only_flag as u64;
        } else {
            let mut sub_wc = 1;
            let mut sub_hc = 2;

            if chroma_format_idc == 3 {
                sub_wc = 2
            } 

            if chroma_format_idc == 1 {
                sub_hc = 1;
            } 

            crop_unit_x = sub_wc;
            crop_unit_y = sub_hc * (2 - frame_mbs_only_flag as u64)
        }
        
        let mut codec_width = (pic_width_in_mbs_minus1 + 1) * 16;
        let mut codec_height = (2 - frame_mbs_only_flag as u64) * ((pic_height_in_map_units_minus1 + 1) * 16);
        codec_width -= (frame_crop_left_offset + frame_crop_right_offset) * crop_unit_x;
        codec_height -= (frame_crop_top_offset + frame_crop_bottom_offset) * crop_unit_y;
    
        let presentation_width = (codec_width * sar_width + (sar_height - 1)); // sar_height
        let presentation_height = codec_height;

        self.width = presentation_width as u32;
        self.height = presentation_height as u32;

        // println!("codec_width{}, codec_height{}", codec_width, codec_height);
        // println!("presentation_width{}, presentation_height{}", presentation_width, presentation_height);

        // println!("=========================");

        Ok(())
    }
}

impl TryFrom<&[u8]> for DecoderConfigurationRecord {
    type Error = AvcError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // FIXME: add checks before accessing buf, otherwise could panic
        let mut buf = Cursor::new(Bytes::copy_from_slice(bytes));

        if buf.remaining() < 7 {
            return Err(AvcError::NotEnoughData("AVC configuration record"));
        }

        let version = buf.read_u8()?;
        if version != 1 {
            return Err(AvcError::UnsupportedConfigurationRecordVersion(version));
        }

        let profile_indication = buf.read_u8()?;
        let profile_compatability = buf.read_u8()?;
        let level_indication = buf.read_u8()?;
        let nalu_size = (buf.read_u8()? & 0x03) + 1;

        let sps_count = buf.read_u8()? & 0x1F;
        let mut sps = Vec::new();
        for _ in 0..sps_count {
            if buf.remaining() < 2 {
                return Err(AvcError::NotEnoughData("DCR SPS length"));
            }
            let sps_length = buf.read_u16::<BigEndian>()? as usize;

            if buf.remaining() < sps_length {
                return Err(AvcError::NotEnoughData("DCR SPS data"));
            }
            let tmp = buf.chunk()[..sps_length].to_owned();
            buf.advance(sps_length);

            sps.push(nal::Unit::try_from(&*tmp)?);
        }

        let pps_count = buf.read_u8()?;
        let mut pps = Vec::new();
        for _ in 0..pps_count {
            if buf.remaining() < 2 {
                return Err(AvcError::NotEnoughData("DCR PPS length"));
            }
            let pps_length = buf.read_u16::<BigEndian>()? as usize;

            if buf.remaining() < pps_length {
                return Err(AvcError::NotEnoughData("DCR PPS data"));
            }
            let tmp = buf.chunk()[..pps_length].to_owned();
            buf.advance(pps_length);

            pps.push(nal::Unit::try_from(&*tmp)?);
        }

        // It turns out that sometimes the extended config is not present, even though the avc_profile_indication
        // is not 66, 77 or 88. We need to be lenient here on decoding.
        let extended_config = match profile_indication {
            66 | 77 | 88 => None,
            _ => {
                if buf.has_remaining() {
                    let chroma_format = buf.read_u8()? & 0b00000011; // 2 bits (6 bits reserved)
                    let bit_depth_luma_minus8 = buf.read_u8()? & 0b00000111; // 3 bits (5 bits reserved)
                    let bit_depth_chroma_minus8 = buf.read_u8()? & 0b00000111; // 3 bits (5 bits reserved)
                    let number_of_sequence_parameter_set_ext = buf.read_u8()?; // 8 bits

                    let mut sequence_parameter_set_ext =
                        Vec::with_capacity(number_of_sequence_parameter_set_ext as usize);
                    for _ in 0..number_of_sequence_parameter_set_ext {
                        let sps_ext_length = buf.read_u16::<BigEndian>()?;
                        let sps_ext_data = buf.read_slice(sps_ext_length as usize)?;
                        sequence_parameter_set_ext.push(sps_ext_data);
                    }

                    Some(AvccExtendedConfig {
                        chroma_format,
                        bit_depth_luma_minus8,
                        bit_depth_chroma_minus8,
                        //sequence_parameter_set_ext,
                    })
                } else {
                    // No extended config present even though avc_profile_indication is not 66, 77 or 88
                    None
                }
            }
        };

        Ok(Self {
            version,
            profile_indication,
            profile_compatability,
            level_indication,
            nalu_size,
            sps,
            pps,
            color_config: None,
            width: 0,
            height: 0,
        })
    }
}

impl DecoderConfigurationRecord {
    pub fn ready(&self) -> bool {
        !self.sps.is_empty() && !self.pps.is_empty()
    }
}
