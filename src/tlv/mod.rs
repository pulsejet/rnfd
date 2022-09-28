pub mod stream_decode;
pub mod vec_decode;

#[derive(Debug)]
pub struct TLV<'s> {
    pub t: u64,
    pub l: u64,
    pub v: &'s [u8],
}

#[derive(Debug)]
pub struct TLO {
    pub t: u64,
    pub l: u64,
    pub o: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Type {
    // 0 Unassigned
    ImplicitSha256DigestComponent = 0x01,
    // 2 - 4 Unassigned
    Interest = 0x05,
    Data = 0x06,
    Name = 0x07,
    NameComponent = 0x08,
    Selectors = 0x09,
    Nonce = 0x0a,
    // 11 (0x0b) Reserved (formely Scope)
    InterestLifetime = 0x0c,
    MinSuffixComponents = 0x0d,
    MaxSuffixComponents = 0x0e,
    PublisherPublicKeyLocator = 0x0f,
    Exclude = 0x10,
    ChildSelector = 0x11,
    MustBeFresh = 0x12,
    Any = 0x13,
    MetaInfo = 0x14,
    Content = 0x15,
    SignatureInfo = 0x16,
    SignatureValue = 0x17,
    ContentType = 0x18,
    FreshnessPeriod = 0x19,
    FinalBlockId = 0x1a,
    SignatureType = 0x1b,
    KeyLocator = 0x1c,
    KeyDigest = 0x1d,
}
