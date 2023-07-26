use anyhow::Result;
use bytesio::bit_reader::BitReader;

const HEADER_SIZE:u8 = 6;

fn read_pes_timming(mut packet: BitReader) -> Result<(u64, u64)> {
    let flag = packet.read_bits(8)? as u8;
    let mut pts:u64 = 0;
    let mut dts:u64 = 0;

    packet.seek_bits(8)?;

    if (flag & 0xC0) != 0 {
        pts = ((u64::from(packet.read_bits(8)?) & 0x0E) << 29)
            | (u64::from(packet.read_bits(8)?) << 22)
            | (u64::from(packet.read_bits(8)? & 0xFF) << 14)
            | (u64::from(packet.read_bits(8)?) << 7)
            | u64::from((packet.read_bits(8)? & 0xFF) >> 1);

        dts = pts;

        if (flag & 0x40) != 0 {
            dts = ((u64::from(packet.read_bits(8)?) & 0x0E) << 29)
            | (u64::from(packet.read_bits(8)?) << 22)
            | (u64::from(packet.read_bits(8)? & 0xFF) << 14)
            | (u64::from(packet.read_bits(8)?) << 7)
            | u64::from((packet.read_bits(8)? & 0xFF) >> 1);


            dts = 0;
        }
    }

    Ok((pts, dts))
}
