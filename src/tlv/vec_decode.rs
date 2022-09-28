use super::TLO;

pub fn read_varnumber(vec: &[u8]) -> Result<(u64, usize), std::io::Error> {
    if vec.len() < 1 {
        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));
    }

    let first = vec[0];

    if first <= 252 {
        return Ok((u64::from(first), 1));
    } else if first == 253 {
        if vec.len() < 3 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));
        }
        return Ok((u64::from(u16::from_be_bytes([vec[1], vec[2]])), 3));
    } else if first == 254 {
        if vec.len() < 5 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));
        }
        return Ok((u64::from(u32::from_be_bytes([vec[1], vec[2], vec[3], vec[4]])), 5));
    } else if first == 255 {
        if vec.len() < 9 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));
        }
        return Ok((u64::from(u64::from_be_bytes([
            vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7], vec[8],
        ])), 7));
    }

    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid varnumber",
    ));
}

pub fn read_nni(vec: &[u8], len: u64) -> Result<u64, std::io::Error> {
    if vec.len() < len as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Insufficient length buffer",
        ));
    }

    if len == 1 {
        return Ok(u64::from(vec[0]));
    } else if len == 2 {
        return Ok(u64::from(u16::from_be_bytes([vec[0], vec[1]])));
    } else if len == 4 {
        return Ok(u64::from(u32::from_be_bytes([vec[0], vec[1], vec[2], vec[3]])));
    } else if len == 8 {
        return Ok(u64::from(u64::from_be_bytes([
            vec[0], vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7],
        ])));
    }

    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid NNI",
    ));
}

pub fn read_u8(vec: &[u8]) -> Result<u8, std::io::Error> {
    if vec.len() < 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data",
        ));
    }
    return Ok(vec[0]);
}

pub fn read_u16(vec: &[u8]) -> Result<u16, std::io::Error> {
    if vec.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data",
        ));
    }
    return Ok(u16::from_be_bytes([vec[0], vec[1]]));
}

pub fn read_u32(vec: &[u8]) -> Result<u32, std::io::Error> {
    if vec.len() < 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data",
        ));
    }
    return Ok(u32::from_be_bytes([
        vec[0], vec[1], vec[2], vec[3],
    ]));
}

pub fn read_u64(vec: &[u8]) -> Result<u64, std::io::Error> {
    if vec.len() < 8 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data",
        ));
    }
    return Ok(u64::from_be_bytes([
        vec[0], vec[1], vec[2], vec[3], vec[4], vec[5], vec[6], vec[7],
    ]));
}

pub fn read_tlo(vec: &[u8]) -> Result<TLO, std::io::Error> {
    let (t, l_t) = read_varnumber(&vec[..])?;
    let (l, l_l) = read_varnumber(&vec[l_t..])?;
    let o = l_t+l_l;
    Ok(TLO { t, l, o })
}