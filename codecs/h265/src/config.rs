use {
    bytes::Buf,
    bytes::BufMut,
    std::{convert::TryFrom, io::Cursor},
};

use std::io::{Read, self, Write, Seek};
use byteorder::{WriteBytesExt, LittleEndian, BigEndian, ReadBytesExt};
use bytes::{Bytes, BytesMut};
use bytesio::{bit_writer::BitWriter, bit_reader::BitReader};
use exp_golomb::{read_exp_golomb, read_signed_exp_golomb};

use crate::{error::HevcError, nal::{self, NaluType}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HEVCDecoderConfigurationRecord {
    pub configuration_version: u8,
    pub general_profile_space: u8,
    pub general_tier_flag: bool,
    pub general_profile_idc: u8,
    pub general_profile_compatibility_flags: u32,
    pub general_constraint_indicator_flags: u64,
    pub general_level_idc: u8,
    pub chroma_format_idc: u8,
    pub bit_depth_luma_minus8: u8,
    pub bit_depth_chroma_minus8: u8,

    pub avg_frame_rate: u16,
    pub constant_frame_rate: u8,
    pub length_size_minus_one: u8,
    pub min_spatial_segmentation_idc:u16,

    //PPS
    pub num_temporal_layers: u8,
    pub temporal_id_nested: bool,

    //VPS
    pub parallelism_type: u8,

    pub vps: Vec<nal::Unit>,
    pub sps: Vec<nal::Unit>,
    pub pps: Vec<nal::Unit>,
}

impl Default for HEVCDecoderConfigurationRecord {
    fn default() -> Self {
        Self {
            configuration_version: 1u8,

            general_profile_space: Default::default(),
            general_tier_flag: Default::default(),
            general_profile_idc: Default::default(),
            general_profile_compatibility_flags: 0xffffffff,
            general_constraint_indicator_flags: 0xffffffffffff,
            general_level_idc: Default::default(),

            chroma_format_idc: Default::default(),
            bit_depth_luma_minus8: Default::default(),
            bit_depth_chroma_minus8: Default::default(),

            
            avg_frame_rate: Default::default(),
            constant_frame_rate: Default::default(),
            length_size_minus_one: 3u8,
            min_spatial_segmentation_idc: Default::default(),

            //PPS
            num_temporal_layers: Default::default(),
            temporal_id_nested: Default::default(),

            //VPS
            parallelism_type:Default::default(),

            vps: Default::default(),
            sps: Default::default(),
            pps: Default::default(),
        }
    }
}

impl TryFrom<&[u8]> for HEVCDecoderConfigurationRecord {
    type Error = HevcError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut buf = Cursor::new(bytes);
        if buf.remaining() < 27 {
            return Err(HevcError::NotEnoughData("AVC configuration record"));
        }
        let configuration_version = buf.get_u8();
        if configuration_version != 1 {
            return Err(HevcError::UnsupportedConfigurationRecordVersion(
                configuration_version,
            ));
        }

        buf.advance(22);

        if buf.get_u8() & 0x3f != NaluType::NaluTypeVps as u8 {
            return Err(HevcError::NotEnoughData("DCR Vps length"));
        }

        let num_nalus = buf.get_u16();

        let mut vps = Vec::new();
        for _ in 0..num_nalus {
            if buf.remaining() < 2 {
                return Err(HevcError::NotEnoughData("DCR Vps length"));
            }
            let vps_length = buf.get_u16() as usize;

            if buf.remaining() < vps_length {
                return Err(HevcError::NotEnoughData("DCR Vps data"));
            }
            let tmp = buf.chunk()[..vps_length].to_owned();
            buf.advance(vps_length);

            vps.push(nal::Unit::try_from(&*tmp)?);
        }

        if buf.get_u8() & 0x3f != NaluType::NaluTypeSps as u8 {
            return Err(HevcError::NotEnoughData("DCR SPS length"));
        }

        let sps_count = buf.get_u16();
        let mut sps = Vec::new();
        for _ in 0..sps_count {
            if buf.remaining() < 2 {
                return Err(HevcError::NotEnoughData("DCR SPS length"));
            }
            let sps_length = buf.get_u16() as usize;

            if buf.remaining() < sps_length {
                return Err(HevcError::NotEnoughData("DCR SPS data"));
            }
            let tmp = buf.chunk()[..sps_length].to_owned();
            buf.advance(sps_length);

            sps.push(nal::Unit::try_from(&*tmp)?);
        }

        if buf.get_u8() & 0x3f != NaluType::NaluTypePps as u8 {
            return Err(HevcError::NotEnoughData("DCR SPS length"));
        }

        let pps_count = buf.get_u16();
        let mut pps = Vec::new();
        for _ in 0..pps_count {
            if buf.remaining() < 2 {
                return Err(HevcError::NotEnoughData("DCR PPS length"));
            }
            let pps_length = buf.get_u16() as usize;

            if buf.remaining() < pps_length {
                return Err(HevcError::NotEnoughData("DCR PPS data"));
            }
            let tmp = buf.chunk()[..pps_length].to_owned();
            buf.advance(pps_length);

            pps.push(nal::Unit::try_from(&*tmp)?);
        }

        let mut c = Self::default();

        c.configuration_version = configuration_version;
        c.vps = vps;
        c.sps = sps;
        c.pps = pps;

        Ok(c)
    }
}

impl HEVCDecoderConfigurationRecord {
    pub fn parse(&mut self) -> Result<(), HevcError> {
        self.parse_sps()?;
        self.parse_vps()?;
        self.parse_pps()?;
        
        Ok(())
    }

    fn parse_vps(&mut self) -> Result<(), HevcError> {
        let vps = &self.vps[0].data;
        let buffer = HEVCDecoderConfigurationRecord::ebsp_to_rbsp(vps);
        let mut bit_reader = BitReader::from(buffer);

        let _video_parameter_set_id = bit_reader.read_bits(4)?;
        bit_reader.seek_bits(2)?;
        let _max_layers_minus1 = bit_reader.read_bits(6)?;
        let max_sub_layers_minus1 = bit_reader.read_bits(3)?;
        let temporal_id_nesting_flag = bit_reader.read_bit()?;

        let num_temporal_layers = max_sub_layers_minus1 + 1;

        self.num_temporal_layers = num_temporal_layers as u8;
        self.temporal_id_nested = temporal_id_nesting_flag;
        Ok(())
    }

    fn parse_sps(&mut self) -> Result<(), HevcError> {
        let sps = &self.sps[0].data;
        let buffer = HEVCDecoderConfigurationRecord::ebsp_to_rbsp(sps);
        let mut bit_reader = BitReader::from(buffer);

        let video_paramter_set_id = bit_reader.read_bits(4)?;
        let max_sub_layers_minus1 = bit_reader.read_bits(3)?;
        let temporal_id_nesting_flag = bit_reader.read_bit()?;

        self.general_profile_space = bit_reader.read_bits(2)? as u8;
        self.general_tier_flag = bit_reader.read_bit()?;
        self.general_profile_idc =  bit_reader.read_bits(5)? as u8;
        self.general_profile_compatibility_flags = bit_reader.read_u32::<BigEndian>()?;
        self.general_constraint_indicator_flags = bit_reader.read_u48::<BigEndian>()?;

        self.general_level_idc = bit_reader.read_u8()?;
        let mut sub_layer_profile_present_flag: Vec<bool> = Vec::new();
        let mut sub_layer_level_present_flag: Vec<bool> = Vec::new();

        for _ in 0..max_sub_layers_minus1 {
            sub_layer_profile_present_flag.push(bit_reader.read_bit()?);
            sub_layer_level_present_flag.push(bit_reader.read_bit()?);
        }

        if max_sub_layers_minus1 > 0 {
            for _ in max_sub_layers_minus1..8{
                bit_reader.read_bits(2)?;
            } 
        }

        for i in 0..max_sub_layers_minus1 {
            if sub_layer_profile_present_flag[i as usize] {
                bit_reader.read_u8()?;             // sub_layer_profile_space, sub_layer_tier_flag, sub_layer_profile_idc
                bit_reader.read_bits(4)?;   // sub_layer_profile_compatibility_flag
                bit_reader.read_bits(6)?;
            }

            if sub_layer_level_present_flag[i as usize] {
                bit_reader.read_u8()?;
            }
        }

        let _seq_parameter_set_id = read_exp_golomb(&mut bit_reader)?;
        self.chroma_format_idc = read_exp_golomb(&mut bit_reader)? as u8;

        if self.chroma_format_idc == 3{
            bit_reader.read_u8()?;
        }

        let pic_width_in_luma_samples = read_exp_golomb(&mut bit_reader)?;
        let pic_height_in_luma_samples = read_exp_golomb(&mut bit_reader)?;

        let conformance_window_flag = bit_reader.read_bit()?;

        let mut left_offset = 0;
        let mut right_offset = 0;
        let mut top_offset = 0;
        let mut bottom_offset = 0;

        if conformance_window_flag {
            left_offset += read_exp_golomb(&mut bit_reader)?;
            right_offset += read_exp_golomb(&mut bit_reader)?;
            top_offset += read_exp_golomb(&mut bit_reader)?;
            bottom_offset += read_exp_golomb(&mut bit_reader)?;
        }

        self.bit_depth_luma_minus8 = read_exp_golomb(&mut bit_reader)? as u8;
        self.bit_depth_chroma_minus8 = read_exp_golomb(&mut bit_reader)? as u8;

        let width = pic_width_in_luma_samples - (left_offset + right_offset);
        let height = pic_height_in_luma_samples - (top_offset + bottom_offset);

        println!("width {}, height {}", width, height);
            
        Ok(())
    }


    fn parse_pps(&mut self) -> Result<(), HevcError> {
        let pps = &self.pps[0].data;
        let buffer = HEVCDecoderConfigurationRecord::ebsp_to_rbsp(pps);
        let mut bit_reader = BitReader::from(buffer);

        let _pic_parameter_set_id = read_exp_golomb(&mut bit_reader)?;
        let _seq_parameter_set_id = read_exp_golomb(&mut bit_reader)?;
        let _dependent_slice_segments_enabled_flag = bit_reader.read_bit()?;
        let _output_flag_present_flag = bit_reader.read_bit()?;
        let _num_extra_slice_header_bits = bit_reader.read_bits(3)?;
        let _sign_data_hiding_enabled_flag = bit_reader.read_bit()?;
        let _cabac_init_present_flag = bit_reader.read_bit()?;
        let _num_ref_idx_l0_default_active_minus1 = read_exp_golomb(&mut bit_reader)?;
        let _num_ref_idx_l1_default_active_minus1 = read_exp_golomb(&mut bit_reader)?;
        let _init_qp_minus26 = read_signed_exp_golomb(&mut bit_reader)?;
        let _constrained_intra_pred_flag = bit_reader.read_bit()?;
        let _transform_skip_enabled_flag = bit_reader.read_bit()?;
        let cu_qp_delta_enabled_flag = bit_reader.read_bit()?;
        if cu_qp_delta_enabled_flag {
            let _diff_cu_qp_delta_depth = read_exp_golomb(&mut bit_reader)?;
        }
        let _cb_qp_offset = read_signed_exp_golomb(&mut bit_reader)?;
        let _cr_qp_offset = read_signed_exp_golomb(&mut bit_reader)?;
        let _pps_slice_chroma_qp_offsets_present_flag = bit_reader.read_bit()?;
        let _weighted_pred_flag = bit_reader.read_bit()?;
        let _weighted_bipred_flag = bit_reader.read_bit()?;
        let _transquant_bypass_enabled_flag = bit_reader.read_bit()?;
        let tiles_enabled_flag = bit_reader.read_bit()?;
        let entropy_coding_sync_enabled_flag = bit_reader.read_bit()?;
        // and more ...

        // needs hvcC
        self.parallelism_type = 1; // slice-based parallel decoding
        if entropy_coding_sync_enabled_flag && tiles_enabled_flag {
            self.parallelism_type = 0; // mixed-type parallel decoding
        } else if entropy_coding_sync_enabled_flag {
            self.parallelism_type = 3; // wavefront-based parallel decoding
        } else if tiles_enabled_flag {
            self.parallelism_type = 2; // tile-based parallel decoding
        }


        Ok(())
    }

    // ISO/IEC-14496-10-2022 - 3.1.48
    fn ebsp_to_rbsp(ebsp_data: &Bytes) -> Bytes {
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
    

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];

        buf.put_u8(self.configuration_version);

        buf.put_u8(
            self.general_profile_space << 6
                | (self.general_tier_flag as u8) << 5
                | self.general_profile_idc,
        );

        buf.put_u32(self.general_profile_compatibility_flags);
        buf.put_u32((self.general_constraint_indicator_flags >> 16) as u32);
        buf.put_u16((self.general_constraint_indicator_flags) as u16);
        buf.put_u8(self.general_level_idc);

        // pub min_spatial_segmentation_idc:u16,
        buf.put_u16(0xf000);
        // pub parallelism_type: u8,
        buf.put_u8(0xfc);

        buf.put_u8(self.chroma_format_idc | 0xfc);

        buf.put_u8(self.bit_depth_luma_minus8 | 0xf8);
        buf.put_u8(self.bit_depth_chroma_minus8 | 0xf8);

        //avg_frame_rate
        buf.put_u16(0);

        buf.put_u8(
            0 << 6
                | self.num_temporal_layers << 3
                | (self.temporal_id_nested as u8) << 2
                | self.length_size_minus_one,
        );

        buf.put_u8(0x03);

        //vps
        buf.put_u8(32u8);
        buf.put_u16(1);
        let temp: Vec<u8> = (&self.vps[0]).into();
        buf.put_u16(temp.len() as u16);
        buf.extend_from_slice(temp.as_slice());

        //sps
        buf.put_u8(33u8);
        buf.put_u16(1);
        let temp: Vec<u8> = (&self.sps[0]).into();
        buf.put_u16(temp.len() as u16);
        buf.extend_from_slice(temp.as_slice());

        //pps
        buf.put_u8(34u8);
        buf.put_u16(1);
        let temp: Vec<u8> = (&self.pps[0]).into();
        buf.put_u16(temp.len() as u16);
        buf.extend_from_slice(temp.as_slice());

        buf
    }

    pub fn demux(reader: &mut io::Cursor<Bytes>) -> io::Result<Self> {
        Ok(HEVCDecoderConfigurationRecord::try_from(reader.chunk()).unwrap())
    }

    pub fn mux<T: io::Write>(&self, writer: &mut T) -> io::Result<()> {
        let mut bit_writer = BitWriter::default();

        bit_writer.write_u8(self.configuration_version)?;
        bit_writer.write_bits(self.general_profile_space as u64, 2)?;
        bit_writer.write_bit(self.general_tier_flag)?;
        bit_writer.write_bits(self.general_profile_idc as u64, 5)?;
        bit_writer.write_u32::<LittleEndian>(self.general_profile_compatibility_flags)?;
        bit_writer.write_u48::<LittleEndian>(self.general_constraint_indicator_flags)?;
        bit_writer.write_u8(self.general_level_idc)?;

        bit_writer.write_bits(0b1111, 4)?; // reserved_4bits
        bit_writer.write_bits(self.min_spatial_segmentation_idc as u64, 12)?;

        bit_writer.write_bits(0b111111, 6)?; // reserved_6bits
        bit_writer.write_bits(self.parallelism_type as u64, 2)?;

        bit_writer.write_bits(0b111111, 6)?; // reserved_6bits
        bit_writer.write_bits(self.chroma_format_idc as u64, 2)?;

        bit_writer.write_bits(0b11111, 5)?; // reserved_5bits
        bit_writer.write_bits(self.bit_depth_luma_minus8 as u64, 3)?;

        bit_writer.write_bits(0b11111, 5)?; // reserved_5bits
        bit_writer.write_bits(self.bit_depth_chroma_minus8 as u64, 3)?;

        bit_writer.write_u16::<BigEndian>(self.avg_frame_rate)?;
        bit_writer.write_bits(self.constant_frame_rate as u64, 2)?;

        bit_writer.write_bits(self.num_temporal_layers as u64, 3)?;
        bit_writer.write_bit(self.temporal_id_nested)?;
        bit_writer.write_bits(self.length_size_minus_one as u64, 2)?;

        // Number of arrays NALs (SPS, PPS, VPS)
        bit_writer.write_u8(0x03)?;

        //vps
        bit_writer.write_u8(32u8)?;
        bit_writer.write_u16::<BigEndian>(1)?;
        let temp: Vec<u8> = (&self.vps[0]).into();
        bit_writer.write_u16::<BigEndian>(temp.len() as u16)?;
        bit_writer.write_all(temp.as_slice())?;

        //sps
        bit_writer.write_u8(33u8)?;
        bit_writer.write_u16::<BigEndian>(1)?;
        let temp: Vec<u8> = (&self.sps[0]).into();
        bit_writer.write_u16::<BigEndian>(temp.len() as u16)?;
        bit_writer.write_all(temp.as_slice())?;

        //pps
        bit_writer.write_u8(34u8)?;
        bit_writer.write_u16::<BigEndian>(1)?;
        let temp: Vec<u8> = (&self.pps[0]).into();
        bit_writer.write_u16::<BigEndian>(temp.len() as u16)?;
        bit_writer.write_all(temp.as_slice())?;

        writer.write_all(&bit_writer.into_inner())?;

        Ok(())
    }

    pub fn size(&self) -> u64 {
        1 // configuration_version
        + 1 // general_profile_space, general_tier_flag, general_profile_idc
        + 4 // general_profile_compatibility_flags
        + 6 // general_constraint_indicator_flags
        + 1 // general_level_idc
        + 2 // reserved_4bits, min_spatial_segmentation_idc
        + 1 // reserved_6bits, parallelism_type
        + 1 // reserved_6bits, chroma_format_idc
        + 1 // reserved_5bits, bit_depth_luma_minus8
        + 1 // reserved_5bits, bit_depth_chroma_minus8
        + 2 // avg_frame_rate
        + 1 // constant_frame_rate, num_temporal_layers, temporal_id_nested, length_size_minus_one
        + 1 // num_of_arrays
        + 1
        + self.sps.iter().map(|sps| {
            2 // sps_length
            + sps.payload().len() as u64
        }).sum::<u64>()
        + 1
        + self.pps.iter().map(|pps| {
            2 // sps_length
            + pps.payload().len() as u64
        }).sum::<u64>()
        + 1
        + self.vps.iter().map(|vps| {
            2 // sps_length
            + vps.payload().len() as u64
        }).sum::<u64>()

    }

}
