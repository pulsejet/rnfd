// Type length value struct
#[derive(Debug, Clone)]
pub struct TLV {
    t: u64,
    l: u64,
    v: Vec<u8>,
}

impl TLV {
    pub fn new(t: u64, l: u64, v: Vec<u8>) -> Self { Self { t, l, v } }
}