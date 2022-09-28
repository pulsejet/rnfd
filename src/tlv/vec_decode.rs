use super::TLO;

/*
 * If the first octet is less than or equal to 252 (0xFC), the number is encoded in that octet.
 * If the first octet is 253 (0xFD), the number is encoded in the following 2 octets, in network byte-order. This number must be greater than 252 (0xFC).
 * If the first octet is 254 (0xFE), the number is encoded in the following 4 octets, in network byte-order. This number must be greater than 65535 (0xFFFF).
 * If the first octet is 255 (0xFF), the number is encoded in the following 8 octets, in network byte-order. This number must be greater than 4294967295 (0xFFFFFFFF).
*/
fn read_varnumber(vec: &[u8]) -> Result<(u64, usize), std::io::Error> {
    let first = vec[0];

    if first <= 252 {
        return Ok((u64::from(first), 1));
    } else if first == 253 {
        return Ok((u64::from(u16::from_be_bytes([vec[1], vec[2]])), 3));
    } else if first == 254 {
        return Ok((u64::from(u32::from_be_bytes([vec[1], vec[2], vec[3], vec[4]])), 5));
    } else if first == 255 {
        return Ok((u64::from(u64::from_be_bytes([
            vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7], vec[8],
        ])), 7));
    }

    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid varnumber",
    ));
}

pub fn read_tlo(vec: &[u8]) -> Result<TLO, std::io::Error> {
    let (t, l_t) = read_varnumber(&vec[..])?;
    let (l, l_l) = read_varnumber(&vec[l_t..])?;
    let o = l_t+l_l;
    Ok(TLO { t, l, o })
}