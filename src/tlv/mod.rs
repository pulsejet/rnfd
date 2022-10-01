pub mod vec_decode;
pub mod varnumber;

#[derive(Debug)]
pub struct TLO {
    pub t: u64,
    pub l: u64,
    pub o: usize,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
#[repr(u64)]
pub enum Type {
    Invalid                         = 0,
    Interest                        = 5,
    Data                            = 6,
    Name                            = 7,
    GenericNameComponent            = 8,
    ImplicitSha256DigestComponent   = 1,
    ParametersSha256DigestComponent = 2,
    CanBePrefix                     = 33,
    MustBeFresh                     = 18,
    ForwardingHint                  = 30,
    Nonce                           = 10,
    InterestLifetime                = 12,
    HopLimit                        = 34,
    ApplicationParameters           = 36,
    InterestSignatureInfo           = 44,
    InterestSignatureValue          = 46,
    MetaInfo                        = 20,
    Content                         = 21,
    SignatureInfo                   = 22,
    SignatureValue                  = 23,
    ContentType                     = 24,
    FreshnessPeriod                 = 25,
    FinalBlockId                    = 26,
    SignatureType                   = 27,
    KeyLocator                      = 28,
    KeyDigest                       = 29,
    SignatureNonce                  = 38,
    SignatureTime                   = 40,
    SignatureSeqNum                 = 42,
}
