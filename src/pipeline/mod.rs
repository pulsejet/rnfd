use crate::tlv;

pub mod incoming;
pub mod interest;

#[derive(Debug)]
pub struct Interest {
    pub name: Vec<u8>,
    pub can_be_prefix: Option<bool>,
    pub must_be_fresh: Option<bool>,
    pub nonce: Option<u32>,
    pub lifetime: Option<u64>,
    pub hop_limit: Option<u8>,
    pub outer_tlo: tlv::TLO,
}
impl Interest {
    pub fn new(name: Vec<u8>, o_tlo: tlv::TLO) -> Interest {
        Interest {
            name: name,
            can_be_prefix: None,
            must_be_fresh: None,
            nonce: None,
            lifetime: None,
            hop_limit: None,
            outer_tlo: o_tlo,
        }
    }
}
