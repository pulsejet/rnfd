use std::ops;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct VarNumber {
    inner: Vec<u8>,
    value: u64,
}

impl VarNumber {
    pub fn length(&self) -> usize {
        self.inner.len()
    }

    fn from_u64(value: u64) -> Self {
        let mut inner = Vec::new();
        if value <= 252 {
            inner.push(value as u8);
        } else if value <= 65535 {
            inner.push(253);
            inner.extend_from_slice(&value.to_be_bytes()[6..]);
        } else if value <= 4294967295 {
            inner.push(254);
            inner.extend_from_slice(&value.to_be_bytes()[4..]);
        } else {
            inner.push(255);
            inner.extend_from_slice(&value.to_be_bytes());
        }
        Self { inner, value }
    }

    fn as_u64(&self) -> u64 {
        self.value
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.clone()
    }
}

impl ops::Add for VarNumber {
    type Output = VarNumber;

    fn add(self, rhs: VarNumber) -> Self::Output {
        VarNumber::from_u64(self.as_u64() + rhs.as_u64())
    }
}

impl ops::Add<u64> for VarNumber {
    type Output = VarNumber;

    fn add(self, rhs: u64) -> Self::Output {
        VarNumber::from_u64(self.as_u64() + rhs)
    }
}

impl From<u8> for VarNumber {
    #[inline]
    fn from(u: u8) -> Self {
        VarNumber::from_u64(u64::from(u))
    }
}

impl From<u16> for VarNumber {
    #[inline]
    fn from(u: u16) -> Self {
        VarNumber::from_u64(u64::from(u))
    }
}

impl From<u32> for VarNumber {
    #[inline]
    fn from(u: u32) -> Self {
        VarNumber::from_u64(u64::from(u))
    }
}

impl From<u64> for VarNumber {
    #[inline]
    fn from(u: u64) -> Self {
        VarNumber::from_u64(u)
    }
}

impl From<usize> for VarNumber {
    #[inline]
    fn from(u: usize) -> Self {
        VarNumber::from_u64(u as u64)
    }
}
