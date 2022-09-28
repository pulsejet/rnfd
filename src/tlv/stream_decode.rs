use std::{io::Read};

use super::TLV;

/*
 * If the first octet is less than or equal to 252 (0xFC), the number is encoded in that octet.
 * If the first octet is 253 (0xFD), the number is encoded in the following 2 octets, in network byte-order. This number must be greater than 252 (0xFC).
 * If the first octet is 254 (0xFE), the number is encoded in the following 4 octets, in network byte-order. This number must be greater than 65535 (0xFFFF).
 * If the first octet is 255 (0xFF), the number is encoded in the following 8 octets, in network byte-order. This number must be greater than 4294967295 (0xFFFFFFFF).
*/
fn read_varnumber(stream: &mut impl Read) -> Result<u64, std::io::Error> {
    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;
    let first = buf[0];

    if first <= 252 {
        return Ok(u64::from(first));
    } else if first == 253 {
        let mut buf = [0u8; 2];
        stream.read_exact(&mut buf)?;
        return Ok(u64::from(u16::from_be_bytes(buf)));
    } else if first == 254 {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf)?;
        return Ok(u64::from(u32::from_be_bytes(buf)));
    } else if first == 255 {
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        return Ok(u64::from(u64::from_be_bytes(buf)));
    }

    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid varnumber",
    ));
}

// pub fn read_tlv(stream: &mut impl Read) -> Result<TLV, std::io::Error> {
//     let t: u64 = read_varnumber(stream)?;
//     let l: u64 = read_varnumber(stream)?;

//     let mut v = vec![0u8; l as usize];
//     stream.read_exact(&mut v)?;

//     Ok(TLV { t, l, v })
// }